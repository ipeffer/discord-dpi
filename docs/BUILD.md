# Сборка на Windows

## Требования

1. [Rust stable](https://rustup.rs/) (`x86_64-pc-windows-msvc`)
2. **Visual Studio Build Tools 2022** с workload **Desktop development with C++**
   - Скачать: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Или: `winget install Microsoft.VisualStudio.2022.BuildTools`
3. WinDivert runtime:

```powershell
.\scripts\setup-windivert.ps1
```

## Сборка

```powershell
$env:WINDIVERT_PATH = "$PWD\vendor\windivert\x64"
cargo build --release -p discdpi-cli
```

## Запуск (от администратора)

```powershell
cargo run -p discdpi-cli -- check
cargo run -p discdpi-cli -- run
```

## Примечание про кириллицу в пути пользователя

Если `x86_64-pc-windows-gnu` падает на линковке, используйте **MSVC toolchain** (по умолчанию в rustup) и установите Build Tools.

CI на GitHub Actions (`windows-latest`) собирает проект автоматически.
