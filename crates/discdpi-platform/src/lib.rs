//! Platform-specific packet capture backends.

#[cfg(windows)]
pub mod windows;

pub trait CaptureBackend {
    fn name(&self) -> &'static str;
}

#[cfg(windows)]
pub use windows::WindowsBackend;

#[cfg(not(windows))]
pub struct WindowsBackend;

#[cfg(not(windows))]
impl CaptureBackend for WindowsBackend {
    fn name(&self) -> &'static str {
        "unsupported"
    }
}
