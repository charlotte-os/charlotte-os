//! # Flanterm Bindings
//!
//! Freestanding bindings to [Flanterm](https://codeberg.org/mintsuki/flanterm), the terminal
//! emulator Catten draws its log output with. The C sources are vendored under `vendor/flanterm`
//! and compiled for the kernel's target by this crate's build script, which also runs bindgen over
//! the public headers, so there is no host toolchain or upstream crate in the loop beyond clang.
//!
//! Everything exported here is a raw, unsafe FFI declaration. `flanterm_context` is opaque by
//! design — Flanterm only hands out pointers to it — so hold the `*mut flanterm_context` returned
//! by [`flanterm_fb_init`] rather than trying to own the value it points at.
//!
//! A context is not internally synchronised; callers must serialise access to it themselves.

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Alignment that satisfies every type Flanterm allocates through the `malloc` callback handed to
/// [`flanterm_fb_init`], mirroring what a hosted `malloc` would guarantee.
///
/// The `free` callback must rebuild its layout with this same alignment.
pub const MALLOC_ALIGN: usize = core::mem::align_of::<max_align_t>();
