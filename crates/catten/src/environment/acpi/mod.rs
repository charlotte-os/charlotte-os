#[doc = include_str!("doc.md")]
pub mod aml;
pub mod sdt;
pub mod table_map;

pub enum Error {
    IrqValOutOfRange,
}
