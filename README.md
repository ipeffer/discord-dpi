# discord-dpi

Open-source DPI bypass tool focused **only on Discord** for Windows. Alternative to [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) with a custom Rust engine (no `winws.exe`).

## Status

Phase 1: WinDivert passthrough capture loop for Discord-related ports.

- `discdpi check` — prerequisites (admin, WinDivert files, profile)
- `discdpi run` — capture and reinject packets (no desync yet)

## Requirements

- Windows 10/11 (64-bit)
- Administrator privileges (WinDivert)
- Secure DNS (DoH) enabled in OS or browser
- [Rust](https://rustup.rs/) stable + Visual Studio Build Tools (C++)

## Setup

```powershell
git clone https://github.com/ipeffer/discord-dpi.git
cd discord-dpi
.\scripts\setup-windivert.ps1
cargo build --release -p discdpi-cli
```

See [docs/BUILD.md](docs/BUILD.md) for detailed build instructions.

## Usage

Run from repository root **as administrator**:

```powershell
cargo run -p discdpi-cli -- check
cargo run -p discdpi-cli -- run
```

Release binary:

```powershell
.\target\release\discdpi.exe check
.\target\release\discdpi.exe run
```

Stop capture with `Ctrl+C`.

## Goals

- Transparent local bypass without VPN or proxy
- Discord desktop app + browser (`discord.com`)
- Voice chat support (UDP 19294–19344, 50000–50100)
- TOML strategy profiles instead of dozens of `.bat` files
- Built-in strategy probe for your ISP

## Project layout

```
crates/
  discdpi-core/      # desync strategy types
  discdpi-filter/    # Discord domain/port filter
  discdpi-platform/  # WinDivert backend (Windows)
  discdpi-cli/       # CLI entrypoint
lists/               # Discord domain/IP lists
profiles/            # TOML strategy profiles
vendor/windivert/    # WinDivert binaries (via setup script)
scripts/             # setup-windivert.ps1
```

## Legal

This project is for circumventing network censorship in jurisdictions where that is legal. Users are responsible for compliance with local laws.

## License

MIT — see [LICENSE](LICENSE). WinDivert is LGPL-3.0.

## Related projects

- [bol-van/zapret](https://github.com/bol-van/zapret) — original DPI bypass engine
- [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) — Windows bundle for Discord/YouTube
- [ValdikSS/GoodbyeDPI](https://github.com/ValdikSS/GoodbyeDPI) — Windows DPI bypass reference
