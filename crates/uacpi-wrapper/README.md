# uacpi-wrapper

Freestanding Rust bindings to [uACPI](https://github.com/uACPI/uACPI), the portable ACPI table
subsystem and AML interpreter Catten drives its firmware interface with.

## Layout

| Path            | Contents                                                             |
| --------------- | -------------------------------------------------------------------- |
| `vendor/uacpi`  | Upstream uACPI **6.1.0**, vendored verbatim (MIT)                     |
| `include.h`     | The translation unit bindgen parses to find uACPI's public surface    |
| `build.rs`      | Compiles the C sources with clang and runs bindgen over `include.h`   |
| `src/lib.rs`    | `no_std` shim that includes the generated bindings                    |

Only `uacpi/*.h` is bound. The headers under `uacpi/internal/` describe the interpreter's own
state and are deliberately left out; `uacpi/platform/` is reached transitively, since the public
headers are written in terms of the `uacpi_*` scalar typedefs it declares.

## The host half of the interface

uACPI is only half a library. Every declaration in `uacpi/kernel_api.h` is a function *the kernel
owes uACPI* — 46 of them at this version — and this crate supplies none of them. Anything that
links it has to define them all with `#[unsafe(no_mangle)]` or the link fails with one undefined
reference per missing function. In this workspace that is
`catten::environment::acpi::uacpi_host`.

## Build requirements

- `clang`, to cross-compile freestanding C for every architecture Catten targets.
- `libclang`, which bindgen loads at runtime (Fedora: `clang-libs`).
- An archiver: `llvm-ar` when it is on `PATH`, otherwise GNU `ar`.

The build script maps each `CARGO_CFG_TARGET_ARCH` onto a bare-metal ELF triple clang understands
(our own target specs under `target_specs/` carry a `catten` environment component that clang does
not recognise) and derives ABI flags matching that spec. No `CC`, `CFLAGS_*` or
`BINDGEN_EXTRA_CLANG_ARGS_*` environment plumbing is needed, though setting `CC`/`AR` still
overrides the defaults.

Every translation unit directly under `vendor/uacpi/source` is compiled, which is how uACPI's own
cmake and meson glue describes its build too, so a release that adds or splits a file does not need
a matching edit here.

uACPI leans on `memcpy`, `memmove`, `memset` and `memcmp`, which the workspace already gets from
`compiler_builtins` via the `compiler-builtins-mem` entry in `.cargo/config.toml`. Everything else
it needs from a libc it implements itself.

## Configuration

uACPI is compiled with its upstream default configuration: the full AML interpreter and event
subsystem, plain (pre-formatted) logging, a `uacpi_kernel_free` that takes no size hint, and
uACPI's own builtin MMIO and zeroed-allocation helpers. There are no Cargo features covering the
`UACPI_*` options, because several of them change the signatures the host half has to match, and a
feature that silently reshaped that contract would surface as a link error at best and a
mismatched ABI at worst. `build.rs` therefore passes no `-D` flags at all; changing the
configuration means adding one there and reworking the matching functions in `uacpi_host`
together, as one deliberate change.

## Updating the vendored sources

1. Replace `vendor/uacpi/include` and `vendor/uacpi/source` with the new release's, along with its
   `LICENSE` and `README.md`, dropping `source/files.cmake` and everything outside those two trees
   (`.github`, `.gitignore`, `tests`, `meson.build`, `meson.options`, `uacpi.cmake`).
2. Update the version in the table above.
3. Rebuild and re-check the host half: `nm --undefined-only` over the built `libuacpi.a` lists
   every symbol the release expects the kernel to provide, which is the authoritative version of
   the list `uacpi_host` implements.
