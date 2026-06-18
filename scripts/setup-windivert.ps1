param(
    [string]$Version = "2.2.2-A"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "vendor\windivert\x64"
$zip = Join-Path $env:TEMP "WinDivert-$Version.zip"
$url = "https://github.com/basil00/WinDivert/releases/download/v2.2.2/WinDivert-$Version.zip"

Write-Host "Downloading WinDivert $Version..."
Invoke-WebRequest -Uri $url -OutFile $zip

$extract = Join-Path $env:TEMP "windivert-extract"
if (Test-Path $extract) { Remove-Item $extract -Recurse -Force }
Expand-Archive -Path $zip -DestinationPath $extract -Force

New-Item -ItemType Directory -Force -Path $dest | Out-Null
$source = Join-Path $extract "WinDivert-$Version\x64"
Copy-Item (Join-Path $source "WinDivert.dll") $dest -Force
Copy-Item (Join-Path $source "WinDivert64.sys") $dest -Force
Copy-Item (Join-Path $source "WinDivert.lib") $dest -Force

Write-Host "Installed WinDivert to $dest"
Get-ChildItem $dest | Format-Table Name, Length
