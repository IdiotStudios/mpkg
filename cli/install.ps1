# Windows Installer for mpkg CLI
# Run as Administrator to install system-wide

$ErrorActionPreference = "Stop"

$binName = "mpkg.exe"
$installDir = "$env:ProgramFiles\mpkg"
$sourcePath = Join-Path -Path (Get-Location) -ChildPath $binName
$targetPath = Join-Path -Path $installDir -ChildPath $binName

# Check if running as admin
If (-Not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "⚠️ You need to run this script as Administrator!" -ForegroundColor Yellow
    Exit 1
}

# Create install directory if it doesn't exist
If (-Not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

# Copy binary to install dir
Copy-Item -Path $sourcePath -Destination $targetPath -Force

Write-Host "✅ mpkg.exe installed to $installDir"

# Add install dir to system PATH if not already present
$envPath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
If ($envPath -notlike "*$installDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$envPath;$installDir", "Machine")
    Write-Host "✅ Added $installDir to system PATH"
} Else {
    Write-Host "ℹ️ $installDir is already in PATH"
}

Write-Host "⚠️ You may need to open a new terminal or restart Windows for PATH changes to take effect"
Write-Host "Try running: mpkg --help"
