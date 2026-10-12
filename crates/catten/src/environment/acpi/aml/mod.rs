use alloc::ffi::CString;

pub mod init;
pub mod interface;
mod uacpi_host;

pub use uacpi_host::initialize;

pub enum Error {
    InvalidPath(CString),
}
