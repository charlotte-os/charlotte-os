# ft-wrapper

Freestanding Rust bindings to [Flanterm](https://codeberg.org/mintsuki/flanterm), the terminal
emulator Catten renders its kernel log with.

This crate replaces the abandoned `flanterm_bindings` crate from crates.io, which pinned an older
Flanterm release and passed x86-only flags (`-mno-red-zone`) and AArch64-only flags
(`-mgeneral-regs-only`) to clang on every non-x86 target, so it could not be built for RISC-V at
all.

## Layout

| Path              | Contents                                                                |
| ----------------- | ----------------------------------------------------------------------- |
| `vendor/flanterm` | Upstream Flanterm **3.1.2**, vendored verbatim (BSD-2-Clause)            |
| `include.h`       | The translation unit bindgen parses to find Flanterm's public surface    |
| `build.rs`        | Compiles the C sources with clang and runs bindgen over `include.h`      |
| `src/lib.rs`      | `no_std` shim that includes the generated bindings                       |

## Build requirements

- `clang`, to cross-compile freestanding C for every architecture Catten targets.
- `libclang`, which bindgen loads at runtime (Fedora: `clang-libs`).
- An archiver: `llvm-ar` when it is on `PATH`, otherwise GNU `ar`.

The build script maps each `CARGO_CFG_TARGET_ARCH` onto a bare-metal ELF triple clang understands
(our own target specs under `target_specs/` carry a `catten` environment component that clang does
not recognise) and derives ABI flags matching that spec. No `CC`, `CFLAGS_*` or
`BINDGEN_EXTRA_CLANG_ARGS_*` environment plumbing is needed, though setting `CC`/`AR` still
overrides the defaults.

## Features

- `bump-alloc` — keep Flanterm's built-in bump allocator. It costs roughly 873 KiB of `.bss` and is
  only ever reached when `flanterm_fb_init` is handed a null `malloc`/`free` pair, so it is off by
  default; Catten supplies the kernel heap instead. With the feature off, passing a null allocator
  pair makes `flanterm_fb_init` return null.

## Updating the vendored sources

1. Replace the contents of `vendor/flanterm` with the new release, dropping its `.github` directory
   and `.gitignore`.
2. Update the version in the table above.
3. Rebuild: any signature change shows up as a compile error in `catten::log::flanterm`, which is
   the only consumer.
