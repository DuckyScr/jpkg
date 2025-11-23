# PowerShell script to install jpkg on Windows

$ErrorActionPreference = "Stop"

$Repo = "DuckyScr/jpkg"
$BinName = "jpkg"
$Archive = "${BinName}-windows-x86_64.zip"
$TmpDir = New-TemporaryFile | Split-Path

Write-Host "Downloading latest nightly release..."
$ReleaseApi = "https://api.github.com/repos/$Repo/releases/latest"
$DownloadUrl = Invoke-RestMethod $ReleaseApi | Select-Object -ExpandProperty assets | Where-Object { $_.name -eq $Archive } | Select-Object -ExpandProperty browser_download_url
Invoke-WebRequest -Uri $DownloadUrl -OutFile "$TmpDir\$Archive"

Write-Host "Extracting..."
Expand-Archive "$TmpDir\$Archive" -DestinationPath "$TmpDir" -Force

Write-Host "Installing to C:\Program Files\$BinName..."
$InstallPath = "C:\Program Files\$BinName"
if (!(Test-Path $InstallPath)) { New-Item -ItemType Directory -Path $InstallPath | Out-Null }
Copy-Item "$TmpDir\$BinName.exe" $InstallPath -Force

# Add to PATH if not already
$EnvPath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
if ($EnvPath -notlike "*$InstallPath*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$EnvPath;$InstallPath", "Machine")
    Write-Host "Added $InstallPath to PATH. You may need to restart your shell."
}

Write-Host "Installed $BinName successfully!"
