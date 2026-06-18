# discord-dpi

Open-source DPI bypass tool focused **only on Discord** for Windows. Alternative to [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) with a custom Rust engine (no `winws.exe`).

## Status

Early scaffold. The WinDivert capture loop and desync engine are not implemented yet.

## Goals

- Transparent local bypass without VPN or proxy
- Discord desktop app + browser (`discord.com`)
- Voice chat support (UDP 19294–19344, 50000–50100)
- TOML strategy profiles instead of dozens of `.bat` files
- Built-in strategy probe for your ISP

## Requirements

- Windows 10/11 (64-bit)
- Administrator privileges (WinDivert)
- Secure DNS (DoH) enabled in OS or browser

## Build

```powershell
cargo build --release
cargo test
```

Run from repository root:

```powershell
cargo run -p discdpi-cli
```

## Project layout

```
crates/
  discdpi-core/      # desync strategy types
  discdpi-filter/    # Discord domain/port filter
  discdpi-platform/  # WinDivert backend (Windows)
  discdpi-cli/       # CLI entrypoint
lists/               # Discord domain/IP lists
profiles/            # TOML strategy profiles
vendor/windivert/    # WinDivert binaries (add manually for now)
```

## Legal

This project is for circumventing network censorship in jurisdictions where that is legal. Users are responsible for compliance with local laws.

## License

MIT — see [LICENSE](LICENSE). WinDivert is LGPL; add `THIRD_PARTY_LICENSES.md` when vendored.

## Related projects

- [bol-van/zapret](https://github.com/bol-van/zapret) — original DPI bypass engine
- [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) — Windows bundle for Discord/YouTube
- [ValdikSS/GoodbyeDPI](https://github.com/ValdikSS/GoodbyeDPI) — Windows DPI bypass reference
