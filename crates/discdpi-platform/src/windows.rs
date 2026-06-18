use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use discdpi_core::{DesyncEngine, ProcessOutcome, Profile};
use discdpi_filter::{find_windivert_dir, windivert_filter, windivert_files_present, DiscordFilter};
use windivert::layer::NetworkLayer;
use windivert::packet::WinDivertPacket;
use windivert::prelude::WinDivertShutdownMode;
use windivert::WinDivert;

use super::CaptureBackend;

#[derive(Debug, Default, Clone, Copy)]
pub struct CaptureStats {
    pub received: u64,
    pub sent: u64,
    pub errors: u64,
    pub desynced: u64,
}

pub struct WindowsBackend {
    filter: String,
    engine: DesyncEngine,
    discord_filter: DiscordFilter,
}

impl WindowsBackend {
    pub fn with_profile(profile: &Profile, discord_filter: DiscordFilter) -> anyhow::Result<Self> {
        configure_windivert_runtime()?;

        let engine = profile
            .tcp_stage()
            .map(DesyncEngine::from_tcp_stage)
            .unwrap_or_else(DesyncEngine::inactive);

        Ok(Self {
            filter: windivert_filter(),
            engine,
            discord_filter,
        })
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn run(&mut self) -> anyhow::Result<CaptureStats> {
        tracing::info!(
            filter = %self.filter,
            desync_active = self.engine.is_active(),
            "opening WinDivert handle"
        );

        let handle = WinDivert::<NetworkLayer>::network(&self.filter, 0, Default::default())
            .map_err(|error| anyhow::anyhow!("failed to open WinDivert handle: {error}"))?;

        let handle = Arc::new(Mutex::new(handle));
        let mut stats = CaptureStats::default();
        let mut buffer = [0u8; 65_535];
        let running = Arc::new(AtomicBool::new(true));
        let stop_flag = Arc::clone(&running);
        let shutdown_handle = Arc::clone(&handle);

        ctrlc::set_handler(move || {
            tracing::info!("shutdown requested");
            stop_flag.store(false, Ordering::SeqCst);
            if let Ok(mut guard) = shutdown_handle.lock() {
                if let Err(error) = guard.shutdown(WinDivertShutdownMode::Recv) {
                    tracing::warn!(?error, "failed to shutdown WinDivert recv queue");
                }
            }
        })
        .map_err(|error| anyhow::anyhow!("failed to install Ctrl+C handler: {error}"))?;

        tracing::info!("capture loop started; press Ctrl+C to stop");

        while running.load(Ordering::SeqCst) {
            let packet = {
                let guard = handle
                    .lock()
                    .map_err(|_| anyhow::anyhow!("WinDivert handle lock poisoned"))?;
                guard.recv(Some(&mut buffer))
            };

            match packet {
                Ok(packet) => {
                    stats.received += 1;
                    if stats.received.is_multiple_of(1_000) {
                        tracing::debug!(
                            received = stats.received,
                            sent = stats.sent,
                            desynced = stats.desynced,
                            "capture progress"
                        );
                    }

                    let outcome = self
                        .engine
                        .process(packet.data.as_ref(), &self.discord_filter);

                    let send_result = {
                        let guard = handle
                            .lock()
                            .map_err(|_| anyhow::anyhow!("WinDivert handle lock poisoned"))?;
                        send_outcome(&guard, &packet, outcome)
                    };

                    match send_result {
                        Ok((sent, desynced)) => {
                            stats.sent += sent as u64;
                            if desynced {
                                stats.desynced += 1;
                            }
                        }
                        Err(error) => {
                            stats.errors += 1;
                            tracing::warn!(?error, "failed to reinject packet");
                        }
                    }
                }
                Err(error) => {
                    if running.load(Ordering::SeqCst) {
                        stats.errors += 1;
                        tracing::warn!(?error, "failed to receive packet");
                    }
                }
            }
        }

        tracing::info!(
            received = stats.received,
            sent = stats.sent,
            desynced = stats.desynced,
            errors = stats.errors,
            "capture loop stopped"
        );

        Ok(stats)
    }

    /// Backward-compatible alias for phase 1 naming.
    pub fn run_passthrough(&mut self) -> anyhow::Result<CaptureStats> {
        self.run()
    }
}

fn send_outcome(
    handle: &WinDivert<NetworkLayer>,
    template: &WinDivertPacket<'_, NetworkLayer>,
    outcome: ProcessOutcome,
) -> Result<(u32, bool), windivert::error::WinDivertError> {
    match outcome {
        ProcessOutcome::Passthrough => {
            handle.send(template)?;
            Ok((1, false))
        }
        ProcessOutcome::Modified(packets) => {
            let mut sent = 0u32;
            for bytes in packets {
                let mut packet = template.clone().into_owned();
                packet.data = Cow::Owned(bytes);
                handle.send(&packet)?;
                sent += 1;
            }
            Ok((sent, true))
        }
    }
}

impl CaptureBackend for WindowsBackend {
    fn name(&self) -> &'static str {
        "windivert"
    }
}

pub fn windivert_dir() -> Option<PathBuf> {
    find_windivert_dir()
}

pub fn is_elevated() -> bool {
    #[link(name = "advapi32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn OpenProcessToken(process: isize, access: u32, token: *mut isize) -> i32;
        fn GetTokenInformation(
            token: isize,
            class: u32,
            info: *mut u8,
            length: u32,
            returned: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_ELEVATION: u32 = 20;

    #[repr(C)]
    struct TokenElevation {
        token_is_elevated: u32,
    }

    unsafe {
        let mut token = 0isize;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TokenElevation {
            token_is_elevated: 0,
        };
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TOKEN_ELEVATION,
            (&mut elevation as *mut TokenElevation).cast(),
            std::mem::size_of::<TokenElevation>() as u32,
            &mut size,
        );
        CloseHandle(token);
        ok != 0 && elevation.token_is_elevated != 0
    }
}

fn configure_windivert_runtime() -> anyhow::Result<PathBuf> {
    let dir = find_windivert_dir().ok_or_else(|| {
        anyhow::anyhow!(
            "WinDivert binaries not found. Run scripts/setup-windivert.ps1 from the repository root"
        )
    })?;

    if !windivert_files_present(&dir) {
        anyhow::bail!("WinDivert.dll or WinDivert64.sys missing in {}", dir.display());
    }

    std::env::set_var("WINDIVERT_PATH", &dir);

    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
    if !paths.iter().any(|entry| entry == &dir) {
        paths.insert(0, dir.clone());
        std::env::set_var("PATH", std::env::join_paths(paths)?);
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windivert_filter_is_non_empty() {
        let backend = WindowsBackend {
            filter: windivert_filter(),
            engine: DesyncEngine::inactive(),
            discord_filter: DiscordFilter::default(),
        };
        assert!(!backend.filter().is_empty());
    }
}
