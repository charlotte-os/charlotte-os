//! # uACPI Bindings
//!
//! Freestanding bindings to [uACPI](https://github.com/uACPI/uACPI), the ACPI table subsystem and
//! AML interpreter Catten drives its firmware interface with. The C sources are vendored under
//! `vendor/uacpi` and compiled for the kernel's target by this crate's build script, which also
//! runs bindgen over the public headers, so there is no host toolchain or upstream crate in the
//! loop beyond clang.
//!
//! Everything exported here is a raw, unsafe FFI declaration, generated straight from the headers
//! under `vendor/uacpi/include/uacpi`. Nothing under `uacpi/internal/` is bound: those headers
//! describe the interpreter's own state, not the surface callers are meant to reach for.
//!
//! # The host half of the interface
//!
//! uACPI is only half a library. Every declaration in `uacpi/kernel_api.h` is a function *the
//! kernel owes uACPI* — memory, mapping, locking, timing, PCI and SystemIO access, interrupt
//! installation, deferred work — and this crate deliberately supplies none of them. The bindings
//! below therefore include `extern` declarations that resolve to nothing in `libuacpi.a`; anything
//! that links this crate has to define them itself with `#[unsafe(no_mangle)]`, or the link will
//! fail with an undefined reference for each one that is missing. In this workspace that is
//! `catten::environment::acpi::uacpi_host`.
//!
//! # Configuration
//!
//! uACPI is compiled with its upstream default configuration: the full AML interpreter and event
//! subsystem, plain (pre-formatted) logging, a `uacpi_kernel_free` that takes no size hint, and
//! uACPI's own builtin MMIO and zeroed-allocation helpers. `build.rs` therefore passes no `-D`
//! flags at all, and there are no Cargo features covering the `UACPI_*` options: several of them
//! change the signatures the host half above has to match, and a feature that silently reshaped
//! that contract would surface as a link error at best and a mismatched ABI at worst. Changing
//! the configuration means adding the flag in `build.rs` and reworking the host half to match, as
//! one deliberate change.
//!
//! # Threading
//!
//! uACPI serialises itself through the locks the host hands it, but only once
//! [`uacpi_initialize`] has been called, and only for as long as the host's lock implementations
//! are honest.

#![no_std]
// Everything below the `include!` is machine-generated, so the lints that would
// normally ask for it to be rewritten have nothing to say to anyone.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_op_in_unsafe_fn)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
