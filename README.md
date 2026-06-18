# discord-dpi

Open-source DPI bypass tool focused **only on Discord** for Windows. Alternative to [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) with a custom Rust engine (no `winws.exe`).

## Status

Phase 2: TCP TLS desync for Discord (`fake` + `multisplit` on ClientHello).

- `discdpi check` — prerequisites (admin, WinDivert files, profile)
- `discdpi run` — capture Discord traffic and apply TLS desync

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

## How desync works

Outbound TCP packets to Discord (port 443, SNI in `lists/discord-domains.txt`) are inspected in the WinDivert capture loop. When the payload is a TLS **ClientHello**, the engine may rewrite traffic before reinjecting it:

| Strategy | Effect |
|----------|--------|
| `fake` | Sends one or more decoy copies of the ClientHello with a low TTL so they expire before the ISP DPI sees the real segment. |
| `multisplit` | Splits the ClientHello into two TCP segments (default at byte 1) so DPI reassembly misses the full handshake. |

Configure strategies in `profiles/default.toml`:

```toml
[[stages]]
protocol = "tcp"
ports = ["443"]
desync = ["fake", "multisplit"]

[stages.desync_params]
split_pos = 1
fake_ttl = 2
fake_repeats = 3
```

UDP voice traffic (ports 19294–19344, 50000–50100) is captured but not yet desynced — planned for Phase 3.

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
