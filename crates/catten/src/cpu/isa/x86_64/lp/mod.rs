// x86_64 Logical Processor Operations
pub mod ops;
pub mod thread_context;

pub type LpId = u32;
pub type CoreId = u32;

pub type WiredIntCtlrId = u8;
pub type WiredIntCtlrSrcNum = u8;
pub type IntSrcDscr = u8;
