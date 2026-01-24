# ============================================================
# Rust AutoBuilder for chuniio
# ============================================================

Write-Host "============================================================"
Write-Host "START COMPILING RUST PROJECT"
Write-Host "============================================================"

# Check if Cargo.toml exists
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "[ERR] Cargo.toml not found"
    pause
    exit 1
}

# Build x64 (64-bit) DLL
Write-Host "[1/2] Now Compiling x64 (64-bit) DLL..."
cargo build --release --target x86_64-pc-windows-msvc

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\x86_64-pc-windows-msvc\release\rustio.dll" "rustio_x64.dll" -Force
    Write-Host "[SUCCESS] x64 Done"
} else {
    Write-Host "[FAILURE] x64 failed"
    pause
    exit 1
}

Write-Host ""

# Build x86 (32-bit) DLL
Write-Host "[2/2] Now Compiling x86 (i686) DLL..."
cargo build --release --target i686-pc-windows-msvc

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\i686-pc-windows-msvc\release\rustio.dll" "rustio_x86.dll" -Force
    Write-Host "[SUCCESS] x86 Done"
} else {
    Write-Host "[FAILURE] x86 failed"
    pause
    exit 1
}

Write-Host "============================================================"
Write-Host "All done"
Write-Host "============================================================"
pause