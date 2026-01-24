# ============================================================
# Rust AutoBuilder for aimeio
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

# Build DLL
Write-Host "Now Compiling DLL..."
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\release\rimeio.dll" "rimeio.dll" -Force
    Write-Host "[SUCCESS] Done"
} else {
    Write-Host "[FAILURE] failed"
    pause
    exit 1
}

Write-Host ""
Write-Host "============================================================"
Write-Host "All done"
Write-Host "============================================================"
pause