// x86_64 Logical Processor Operations
pub mod ops;
pub mod thread_context;

pub type LpId = u32;
pub type CoreId = u32;

pub type EicId = u8;
pub type EicPinNum = u8;
pub type IntSrcDscr = u8;
