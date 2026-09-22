use alloc::ffi::CString;

pub mod init;
mod uacpi_host;

pub use uacpi_host::initialize;

pub enum Error {
    InvalidPath(CString),
}
