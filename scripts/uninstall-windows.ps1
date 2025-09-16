param(
  [switch]$Purge
)

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:USERPROFILE ".zirc\bin"

Write-Host "Uninstalling Zirc from" $InstallDir

# Remove from user PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath) {
  $Parts = $CurrentPath -split ";" | Where-Object { $_ -ne "" }
  $NewParts = $Parts | Where-Object { $_ -ne $InstallDir }
  if ($NewParts.Count -ne $Parts.Count) {
    $NewPath = ($NewParts -join ";")
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = $NewPath
    Write-Host "Removed from PATH."
  } else {
    Write-Host "Install dir not present in PATH."
  }
}

# Remove binaries (optional)
if (Test-Path $InstallDir) {
  if ($Purge) {
    Write-Host "Removing" $InstallDir
    Remove-Item -Recurse -Force $InstallDir
  } else {
    Write-Host "Keeping" $InstallDir 
    if (Test-Path (Join-Path $InstallDir "zirc.exe")) { Remove-Item -Force (Join-Path $InstallDir "zirc.exe") }
    if (Test-Path (Join-Path $InstallDir "zirc-repl.exe")) { Remove-Item -Force (Join-Path $InstallDir "zirc-repl.exe") }
  }
}

Write-Host "Uninstall complete. Restart terminals to refresh PATH."
