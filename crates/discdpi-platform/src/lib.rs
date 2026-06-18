//! Platform-specific packet capture backends.

#[cfg(windows)]
pub mod windows;

pub trait CaptureBackend {
    fn name(&self) -> &'static str;
}

#[cfg(windows)]
pub use windows::{CaptureStats, WindowsBackend};

#[cfg(not(windows))]
#[derive(Debug, Default)]
pub struct CaptureStats {
    pub received: u64,
    pub sent: u64,
    pub errors: u64,
}

#[cfg(not(windows))]
pub struct WindowsBackend;

#[cfg(not(windows))]
impl WindowsBackend {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::bail!("discord-dpi capture is only supported on Windows")
    }

    pub fn run_passthrough(&mut self) -> anyhow::Result<CaptureStats> {
        anyhow::bail!("discord-dpi capture is only supported on Windows")
    }
}

#[cfg(not(windows))]
impl CaptureBackend for WindowsBackend {
    fn name(&self) -> &'static str {
        "unsupported"
    }
}

#[cfg(windows)]
pub fn is_elevated() -> bool {
    windows::is_elevated()
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}

#[cfg(windows)]
pub fn windivert_dir() -> Option<std::path::PathBuf> {
    windows::windivert_dir()
}

#[cfg(not(windows))]
pub fn windivert_dir() -> Option<std::path::PathBuf> {
    None
}
