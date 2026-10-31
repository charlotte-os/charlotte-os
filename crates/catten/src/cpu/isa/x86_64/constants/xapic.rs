//! # xAPIC Register Offsets
//!
//! Ref: AMD APM 16.3.2
pub const ID_REG: u16 = 0x20;
pub const EOI_REGISTER: u16 = 0xb0;
pub const LOGICAL_DEST_REG: u16 = 0xd0;
pub const SPURIOUS_INTERRUPT_VECTOR_REG: u16 = 0xf0;
pub const INTERRUPT_COMMAND_REGISTER_LOW: u16 = 0x300;
pub const INTERRUPT_COMMAND_REGISTER_HIGH: u16 = 0x310;
pub const TIMER_LVTR: u16 = 0x320;
pub const TIMER_INITIAL_COUNT: u16 = 0x380;
pub const TIMER_CURRENT_COUNT: u16 = 0x390;
pub const TIMER_DIVIDE_CONFIGURATION: u16 = 0x3e0;
