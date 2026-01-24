# ============================================================
# Generate Build Information Script
# 生成构建信息脚本
# ============================================================

Write-Host "Generating build information..." -ForegroundColor Cyan

$output = @"
=============================================================================
RustIO - CHUNITHM Custom IO DLL Build Information
=============================================================================
Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

-----------------------------------------------------------------------------
[0] Quick Reference
-----------------------------------------------------------------------------

To reproduce this build:
1. Install Rust from https://rustup.rs/
2. Add targets:
   rustup target add x86_64-pc-windows-msvc
   rustup target add i686-pc-windows-msvc
3. Install Visual Studio Build Tools with C++ support
4. Run: .\build.ps1

-----------------------------------------------------------------------------
[1] Rust Toolchain Information
-----------------------------------------------------------------------------
"@

try {
    $rustcVersion = rustc --version 2>&1
    $cargoVersion = cargo --version 2>&1
    $rustupVersion = rustup --version 2>&1| Select-String -NotMatch '^info:'
    
    $output += @"

Rust Compiler:
  $rustcVersion

Cargo:
  $cargoVersion

Rustup:
  $rustupVersion

"@
} catch {
    $output += "`nError: Rust toolchain not found or not in PATH`n"
}

# Installed targets
try {
    $output += @"
-----------------------------------------------------------------------------
[2] Installed Compilation Targets
-----------------------------------------------------------------------------

"@
    $targets = rustup target list --installed 2>&1
    $output += $targets
    $output += "`n"
} catch {
    $output += "Error: Could not retrieve target list`n"
}

# Windows version
try {
    $output += @"

-----------------------------------------------------------------------------
[3] Operating System Information
-----------------------------------------------------------------------------

"@
    $osInfo = Get-WmiObject -Class Win32_OperatingSystem
    $output += "OS Name: $($osInfo.Caption)`n"
    $output += "OS Version: $($osInfo.Version)`n"
    $output += "OS Architecture: $($osInfo.OSArchitecture)`n"
    $output += "Build Number: $($osInfo.BuildNumber)`n"
} catch {
    $output += "Error: Could not retrieve OS information`n"
}

# MSVC information
try {
    $output += @"

-----------------------------------------------------------------------------
[4] Visual Studio / MSVC Information
-----------------------------------------------------------------------------

"@
    
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsInfo = & $vsWhere -latest -property catalog_productDisplayVersion 2>&1
        $output += "Visual Studio Version: $vsInfo"
    } else {
        $output += "Visual Studio Installer not found`n"
    }
} catch {
    $output += "Could not detect Visual Studio installation`n"
}

$output += @"


=============================================================================
End of Build Information
=============================================================================
"@

# Save to file
$output | Out-File -FilePath "BUILD_INFO.txt" -Encoding UTF8

Write-Host ""
Write-Host "✓ Build information saved to BUILD_INFO.txt" -ForegroundColor Green
Write-Host ""
Write-Host "Summary:" -ForegroundColor Yellow
Write-Host "  - Rust Version: $(rustc --version)" -ForegroundColor White
Write-Host "  - Output File: BUILD_INFO.txt" -ForegroundColor White
Write-Host ""

# Also display on screen
Write-Host "Press Enter to view the full report..." -ForegroundColor Cyan
$null = Read-Host
Get-Content "BUILD_INFO.txt"