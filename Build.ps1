# ============================================================
# Rust Workspace AutoBuilder for rimeio & rustio
# ============================================================

Write-Host "============================================================"
Write-Host "START COMPILING RUST WORKSPACE"
Write-Host "============================================================"

if (-not (Test-Path "Cargo.toml")) {
    Write-Host "[ERR] Workspace Cargo.toml not found"
    pause
    exit 1
}

Write-Host "[1/3] Compiling rustio x64..."
cargo build --release --target x86_64-pc-windows-msvc -p rustio

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\x86_64-pc-windows-msvc\release\rustio.dll" "rustio_x64.dll" -Force
    Write-Host "[SUCCESS] rustio_x64.dll updated"
} else {
    Write-Host "[FAILURE] rustio x64 failed"
    pause
    exit 1
}

Write-Host ""

Write-Host "[2/3] Compiling rustio x86..."
cargo build --release --target i686-pc-windows-msvc -p rustio

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\i686-pc-windows-msvc\release\rustio.dll" "rustio_x86.dll" -Force
    Write-Host "[SUCCESS] rustio_x86.dll updated"
} else {
    Write-Host "[FAILURE] rustio x86 failed"
    pause
    exit 1
}

Write-Host ""

Write-Host "[3/3] Compiling rimeio..."
cargo build --release -p rimeio

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\release\rimeio.dll" "rimeio.dll" -Force
    Write-Host "[SUCCESS] rimeio.dll updated"
} else {
    Write-Host "[FAILURE] rimeio failed"
    pause
    exit 1
}

Write-Host "============================================================"
Write-Host "All done"
Write-Host "============================================================"
pause