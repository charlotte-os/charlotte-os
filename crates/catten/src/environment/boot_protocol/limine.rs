//! # Limine Boot Protocol Requests
//!
//! Limine finds these by scanning the loaded executable for their magic numbers, so nothing in the
//! kernel necessarily reads them and the linker is free to discard them as dead. Every static here
//! therefore needs `#[used]` to survive codegen, and a `link_section` that the linker scripts
//! `KEEP`, to survive `--gc-sections`. Dropping one is silent: the request simply never gets a
//! response, and `BASE_REVISION` going missing makes Limine fall back to assuming base revision 0.
//!
//! The start and end markers bracket the requests so Limine can narrow its scan, so they must be
//! placed in their own sections and emitted on either side of the block.

use limine::request::{
    ExecutableAddressRequest,
    FramebufferRequest,
    HhdmRequest,
    MemmapRequest,
    MpRequest,
    RsdpRequest,
    StackSizeRequest,
    TscFrequencyRequest,
};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

use crate::memory::allocators::memory::PageSize;

#[used]
#[unsafe(link_section = ".requests_start_marker")]
pub static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

const MP_X2APIC_ENABLE: u64 = 1 << 0;

#[used]
#[unsafe(link_section = ".requests")]
pub static MP_REQUEST: MpRequest = MpRequest::new(
    if cfg!(target_arch = "x86_64") {
        MP_X2APIC_ENABLE
    } else {
        0
    },
);

#[used]
#[unsafe(link_section = ".requests")]
pub static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static STACK_SIZE: StackSizeRequest =
    StackSizeRequest::new(PageSize::Standard.num_bytes() as u64 * 4);

#[used]
#[unsafe(link_section = ".requests")]
pub static TSC_FREQUENCY_REQUEST: TscFrequencyRequest = TscFrequencyRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
pub static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();
