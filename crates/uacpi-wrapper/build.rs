//! Builds the vendored uACPI C sources for the kernel's target and generates
//! freestanding Rust bindings to its public headers with bindgen.

use std::path::{Path, PathBuf};
use std::{env, fs};

/// Root of the vendored uACPI release, relative to the crate root.
const UACPI_ROOT: &str = "vendor/uacpi";

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH is not set");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let root = Path::new(UACPI_ROOT);
    let include_dir = root.join("include");
    let src_dir = root.join("source");

    let flags = compiler_flags(&arch);

    compile_uacpi(&include_dir, &src_dir, &flags);
    generate_bindings(&include_dir, &flags, &out_dir);

    rerun_if_sources_changed(&include_dir);
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
        other => panic!("uacpi-wrapper does not support the {other} architecture"),
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

    flags
}

fn compile_uacpi(include_dir: &Path, src_dir: &Path, flags: &[String]) {
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
        .include(include_dir);

    // uACPI's build is "every translation unit under source/", which is how its
    // own cmake and meson glue describes it too, so upstream adding or splitting
    // a file across releases does not need a matching edit here.
    for source in c_sources(src_dir) {
        build.file(source);
    }

    for flag in flags {
        build.flag(flag);
    }

    build.compile("uacpi");
}

/// Every `.c` file directly under `src_dir`, in a stable order so that the
/// archive members do not shuffle between builds.
fn c_sources(src_dir: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(src_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", src_dir.display()));

    let mut sources: Vec<PathBuf> = entries
        .map(|entry| entry.expect("failed to read a vendored uACPI directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "c"))
        .collect();

    assert!(!sources.is_empty(), "no C sources found under {}", src_dir.display());

    sources.sort();
    sources
}

fn tool_on_path(name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

fn generate_bindings(include_dir: &Path, flags: &[String], out_dir: &Path) {
    let bindings = bindgen::builder()
        .header("include.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_args(flags)
        // The kernel is `no_std`, so the generated code may only lean on `core`.
        .use_core()
        .ctypes_prefix("::core::ffi")
        .layout_tests(false)
        .derive_debug(false)
        // Keep the generated constants spelled the way the headers spell them:
        // bindgen renders C enums as a type alias plus a set of consts, and by
        // default it prefixes each const with its enum's name, which would turn
        // UACPI_STATUS_OK into uacpi_status_UACPI_STATUS_OK.
        .prepend_enum_name(false)
        // Nothing outside uACPI's own public surface belongs in the bindings:
        // the vendored headers reach `stdint.h` and friends for the scalar
        // typedefs, and those would otherwise drag clang's whole freestanding
        // prelude in with them.
        .allowlist_file(format!("{}/uacpi/.*\\.h", include_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate uACPI bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write the generated uACPI bindings");
}

/// Rebuild whenever any vendored header or translation unit changes.
fn rerun_if_sources_changed(dir: &Path) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));

    for entry in entries {
        let path = entry.expect("failed to read a vendored uACPI directory entry").path();

        if path.is_dir() {
            rerun_if_sources_changed(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
