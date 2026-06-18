use std::path::PathBuf;

use clap::{Parser, Subcommand};
use discdpi_core::Profile;
use discdpi_filter::{find_windivert_dir, windivert_files_present, DiscordFilter};
use discdpi_platform::{is_elevated, windivert_dir, CaptureBackend, WindowsBackend};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "discdpi", about = "Discord-only DPI bypass for Windows")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate runtime prerequisites
    Check {
        #[arg(long, default_value = "profiles/default.toml")]
        profile: PathBuf,
    },
    /// Start WinDivert capture with TLS desync for Discord
    Run {
        #[arg(long, default_value = "profiles/default.toml")]
        profile: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Check { profile } => run_check(&profile),
        Command::Run { profile } => run_capture(&profile),
    }
}

fn run_check(profile_path: &PathBuf) -> anyhow::Result<()> {
    let elevated = is_elevated();
    tracing::info!(elevated, "administrator privileges");

    if let Some(dir) = windivert_dir() {
        tracing::info!(path = %dir.display(), present = windivert_files_present(&dir), "WinDivert runtime");
    } else {
        tracing::warn!("WinDivert runtime not found; run scripts/setup-windivert.ps1");
    }

    let profile_toml = std::fs::read_to_string(profile_path)?;
    let profile = Profile::from_toml(&profile_toml)?;
    tracing::info!(profile = %profile.name.id, stages = profile.stages.len(), "strategy profile");

    if let Some(tcp) = profile.tcp_stage() {
        tracing::info!(
            protocol = %tcp.protocol,
            methods = ?tcp.desync,
            "tcp desync stage"
        );
    }

    let filter = DiscordFilter::load_from_dir(PathBuf::from("lists").as_path())?;
    tracing::info!(
        discord_com = filter.matches_domain("discord.com"),
        voice_port = filter.matches_port(19300),
        "discord filter"
    );

    if !elevated {
        anyhow::bail!("run this program as administrator");
    }

    if find_windivert_dir().is_none() {
        anyhow::bail!("WinDivert binaries are missing");
    }

    Ok(())
}

fn run_capture(profile_path: &PathBuf) -> anyhow::Result<()> {
    if !is_elevated() {
        anyhow::bail!("administrator privileges are required for WinDivert");
    }

    let profile_toml = std::fs::read_to_string(profile_path)?;
    let profile = Profile::from_toml(&profile_toml)?;
    let filter = DiscordFilter::load_from_dir(PathBuf::from("lists").as_path())?;

    let mut backend = WindowsBackend::with_profile(&profile, filter)?;
    tracing::info!(
        profile = %profile.name.id,
        backend = backend.name(),
        "starting capture with TLS desync"
    );

    let stats = backend.run()?;
    tracing::info!(
        received = stats.received,
        sent = stats.sent,
        desynced = stats.desynced,
        errors = stats.errors,
        "session finished"
    );

    Ok(())
}
