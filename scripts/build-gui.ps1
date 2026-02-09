Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

Write-Host "Building CLI (release)..."
cargo build --release -p fag-cli

$src = Join-Path $repoRoot "target\\release\\fag.exe"
$dstDir = Join-Path $repoRoot "apps\\gui\\bin"
$dst = Join-Path $dstDir "fag.exe"

if (!(Test-Path $src)) {
  throw "Missing artifact: $src"
}

New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
Copy-Item -Force $src $dst

Write-Host "Copied: $dst"

