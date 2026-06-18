# Third-party licenses

## WinDivert 2.2.2

- Source: https://github.com/basil00/WinDivert
- License: GNU Lesser General Public License v3.0
- Files: `vendor/windivert/x64/WinDivert.dll`, `WinDivert64.sys`, `WinDivert.lib`
- Installed via `scripts/setup-windivert.ps1` (not committed to git)

## Rust crates

See `Cargo.lock` for the full dependency tree. Key runtime dependencies:

- `windivert` / `windivert-sys` — WinDivert Rust bindings
- `clap` — CLI parsing
- `tracing` — structured logging
