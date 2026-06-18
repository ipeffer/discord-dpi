use super::CaptureBackend;

pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> anyhow::Result<Self> {
        // TODO: initialize WinDivert handle with Discord-only filter
        Ok(Self)
    }
}

impl CaptureBackend for WindowsBackend {
    fn name(&self) -> &'static str {
        "windivert"
    }
}
