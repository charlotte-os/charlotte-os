//! Builds the vendored Flanterm C sources for the kernel's target and generates
//! freestanding Rust bindings to its public headers with bindgen.

use std::path::{Path, PathBuf};
use std::{env, fs};

/// Root of the vendored Flanterm release, relative to the crate root.
const FLANTERM_ROOT: &str = "vendor/flanterm";

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH is not set");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let src_dir = Path::new(FLANTERM_ROOT).join("src");

    let flags = compiler_flags(&arch);

    compile_flanterm(&src_dir, &flags);
    generate_bindings(&src_dir, &flags, &out_dir);

    rerun_if_sources_changed(&src_dir);
    println!("cargo:rerun-if-changed=include.h");
    println!("cargo:rerun-if-changed=build.rs");
}

/// Flags shared by the C build and by bindgen.
///
/// Both have to agree on the target and its ABI, otherwise the type layouts
/// bindgen computes will not match the code clang actually emits.
fn compiler_flags(arch: &str) -> Vec<String> {
    // Catten builds against custom target specs whose triples carry a `catten`
    // environment component that clang knows nothing about, so map the
    // architecture onto a bare-metal ELF triple clang does accept.
    let clang_target = match arch {
        "x86_64" => "x86_64-unknown-none-elf",
        "aarch64" => "aarch64-unknown-none-elf",
        "riscv64" => "riscv64-unknown-none-elf",
        other => panic!("ft-wrapper does not support the {other} architecture"),
    };

    let mut flags = vec![
        format!("--target={clang_target}"),
        // No hosted C library, no stack guards and no lazy PLT binding are
        // available in the kernel, and the workspace links everything statically
        // at fixed addresses.
        String::from("-ffreestanding"),
        String::from("-fno-stack-protector"),
        String::from("-fno-stack-check"),
        String::from("-fno-pic"),
        // The Rust half of the kernel is built with LTO, but the C objects are
        // fed to the linker as a plain archive.
        String::from("-fno-lto"),
    ];

    // Match the ABI and the disabled register files declared by the matching
    // target spec under target_specs/.
    match arch {
        "x86_64" => flags.extend(
            ["-mcmodel=kernel", "-mno-red-zone", "-mno-80387", "-mno-mmx", "-mno-sse", "-mno-sse2"]
                .map(String::from),
        ),
        "aarch64" => flags.extend(["-mgeneral-regs-only", "-mstrict-align"].map(String::from)),
        "riscv64" => {
            flags.extend(["-march=rv64gc", "-mabi=lp64d", "-mcmodel=medany"].map(String::from))
        }
        _ => unreachable!("the architecture was already validated above"),
    }

    // Flanterm's framebuffer backend carries a ~873 KiB bump allocator pool that
    // is only reachable when flanterm_fb_init is handed a null malloc/free pair.
    // Catten always supplies its own allocator, so drop the pool by default.
    if env::var_os("CARGO_FEATURE_BUMP_ALLOC").is_none() {
        flags.push(String::from("-DFLANTERM_FB_DISABLE_BUMP_ALLOC"));
    }

    flags
}

fn compile_flanterm(src_dir: &Path, flags: &[String]) {
    let mut build = cc::Build::new();

    build
        // Cross-compiling freestanding C needs a compiler that can target every
        // architecture the kernel supports; the CC and AR environment variables
        // still take precedence if a different toolchain is preferred.
        .compiler("clang")
        // GNU ar archives foreign-architecture objects just as happily, so it is
        // a fine fallback where the LLVM binutils are not installed.
        .archiver(if tool_on_path("llvm-ar") { "llvm-ar" } else { "ar" })
        .std("c11")
        .pic(false)
        .include(src_dir)
        .file(src_dir.join("flanterm.c"))
        .file(src_dir.join("flanterm_backends/fb.c"));

    for flag in flags {
        build.flag(flag);
    }

    build.compile("flanterm");
}

fn tool_on_path(name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

fn generate_bindings(src_dir: &Path, flags: &[String], out_dir: &Path) {
    let bindings = bindgen::builder()
        .header("include.h")
        .clang_arg(format!("-I{}", src_dir.display()))
        .clang_args(flags)
        // The kernel is `no_std`, so the generated code may only lean on `core`.
        .use_core()
        .ctypes_prefix("::core::ffi")
        .layout_tests(false)
        .derive_debug(false)
        // Flanterm's `#define`s for callback kinds and framebuffer rotations are
        // passed to functions taking `int`.
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate Flanterm bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write the generated Flanterm bindings");
}

/// Rebuild whenever any vendored header or translation unit changes.
fn rerun_if_sources_changed(src_dir: &Path) {
    let entries = fs::read_dir(src_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", src_dir.display()));

    for entry in entries {
        let path = entry.expect("failed to read a vendored Flanterm directory entry").path();

        if path.is_dir() {
            rerun_if_sources_changed(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
