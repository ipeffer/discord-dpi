use std::path::PathBuf;

use discdpi_core::Profile;
use discdpi_filter::DiscordFilter;
use discdpi_platform::{CaptureBackend, WindowsBackend};
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let profile_path = PathBuf::from("profiles/default.toml");
    let profile_toml = std::fs::read_to_string(&profile_path)?;
    let profile = Profile::from_toml(&profile_toml)?;

    let filter = DiscordFilter::load_from_dir(PathBuf::from("lists").as_path())?;
    let backend = WindowsBackend::new()?;

    tracing::info!(profile = %profile.name.id, backend = backend.name(), domains = filter.matches_domain("discord.com"), "discord-dpi scaffold ready");
    tracing::info!("WinDivert capture loop is not implemented yet");
    Ok(())
}
