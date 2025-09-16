param(
  [switch]$Force
)

$ErrorActionPreference = "Stop"

# Resolve workspace root (script is expected under scripts/)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir

$InstallDir = Join-Path $env:USERPROFILE ".zirc\bin"

Write-Host "Installing Zirc CLI to" $InstallDir

# Ensure install dir exists
if (-not (Test-Path $InstallDir)) {
  New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

# Build release binaries
Write-Host "Building release binaries..."
# Build only the zirc-cli crate to speed up
pushd $RepoRoot
try {
  cargo build -p zirc-cli --release
} finally {
  popd
}

# Determine target dir (Rust default)
$TargetDir = Join-Path $RepoRoot "target\release"

# Source exe paths
$ZircExe = Join-Path $TargetDir "zirc.exe"
$ZircReplExe = Join-Path $TargetDir "zirc-repl.exe"

if (-not (Test-Path $ZircExe)) { throw "zirc.exe not found at $ZircExe" }
if (-not (Test-Path $ZircReplExe)) { throw "zirc-repl.exe not found at $ZircReplExe" }

# Copy to install dir
Write-Host "Copying binaries..."
Copy-Item -Force:$Force $ZircExe $InstallDir
Copy-Item -Force:$Force $ZircReplExe $InstallDir

# Add to user PATH if not present
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $CurrentPath) { $CurrentPath = "" }

$InstallDirExpanded = $InstallDir

$PathParts = $CurrentPath -split ";" | Where-Object { $_ -ne "" }

if ($PathParts -notcontains $InstallDirExpanded) {
  Write-Host "Updating user PATH..."
  $NewPath = if ($CurrentPath.Trim().Length -eq 0) { $InstallDirExpanded } else { $CurrentPath.TrimEnd(';') + ";" + $InstallDirExpanded }
  [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
  # Update current session PATH
  $env:Path = $NewPath
  Write-Host "PATH updated."
} else {
  Write-Host "Install directory already on PATH."
}

# Verify
Write-Host "Verifying installation..."
$version = & "$InstallDir\zirc.exe" --version
Write-Host $version

Write-Host "Installation complete. You can now run 'zirc' or 'zirc-repl' from any PowerShell window.",
           "If a currently open terminal doesn't see the command, restart it."