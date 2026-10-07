[CmdletBinding()]
param(
  [switch]$China,
  [string]$Mirror = ""
)

# Set TLS 1.2 protocol for older PowerShell versions
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$repoOwner = "Super1Windcloud"
$repoName = "hyperscoop"
$installDir = "$env:USERPROFILE\Tools\hyperscoop"
$headers = @{ "User-Agent" = "PowerShell" }

$proxyPrefix = ""
if ($China -or ($Mirror -ne "")) {
  $proxyPrefix = if ($Mirror -ne "") { $Mirror.TrimEnd('/') + "/" } else { "https://mirror.ghproxy.com/" }
}

try {
  $apiUrl = if ($proxyPrefix -ne "") { "$proxyPrefix" + "https://api.github.com/repos/$repoOwner/$repoName/releases/latest" } else { "https://api.github.com/repos/$repoOwner/$repoName/releases/latest" }
  $response = Invoke-RestMethod -Uri $apiUrl -Headers $headers
  $version = $response.tag_name
  Write-Host "Latest version: $version"
} catch {
  Write-Error "Get latest version failed: $_"
  return
}

$rawArch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
switch ($rawArch) {
  "AMD64" { $arch = "x64" }
  "ARM64" { $arch = "arm64" }
  default { $arch = "x86" }
}

$assetName = "hp-$arch-$version.exe"
$directDownloadUrl = "https://github.com/$repoOwner/$repoName/releases/download/$version/$assetName"
$downloadUrl = if ($proxyPrefix -ne "") { "$proxyPrefix$directDownloadUrl" } else { $directDownloadUrl }
$checksumUrl = "$downloadUrl.sha256"

if (-not (Test-Path -Path $installDir)) {
  New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$targetPath = Join-Path $installDir "hp.exe"
$checksumPath = "$targetPath.sha256"
try {
  Write-Host "Downloading: $downloadUrl"
  Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath -UseBasicParsing
  Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath -UseBasicParsing
  $expectedHash = ((Get-Content $checksumPath -Raw) -split '\s+')[0].Trim().ToLowerInvariant()
  $actualHash = (Get-FileHash -Path $targetPath -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actualHash -ne $expectedHash) {
    Remove-Item -Path $targetPath -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $checksumPath -Force -ErrorAction SilentlyContinue
    Write-Error "Checksum verification failed. Expected $expectedHash, got $actualHash"
    return
  }
  Remove-Item -Path $checksumPath -Force -ErrorAction SilentlyContinue
  Write-Host "Download complete: $targetPath"
} catch {
  Remove-Item -Path $checksumPath -Force -ErrorAction SilentlyContinue
  Write-Error "Download failed: $_"
  return
}

$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$installDir*") {
  $newPath = if ([string]::IsNullOrWhiteSpace($currentPath)) { $installDir } else { "$currentPath;$installDir" }
  try {
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "Added to User PATH: $installDir"
  } catch {
    Write-Error "Add to User PATH failed: $_"
    return
  }
} else {
  Write-Host "User PATH already contains: $installDir"
}

if ($env:PATH -notlike "*$installDir*") {
  $env:PATH = "$installDir;$env:PATH"
}

# Auto-initialize official 'main' bucket for out-of-the-box readiness
Write-Host "Initializing official 'main' bucket..." -ForegroundColor Cyan
try {
  & "$targetPath" bucket add main
} catch {
  Write-Warning "Could not automatically initialize main bucket: $_"
  Write-Host "You can manually initialize it later via: hp bucket add main" -ForegroundColor Yellow
}

# Optional PowerShell Profile injection for shell environment and completions
try {
  $profileDir = Split-Path -Parent $PROFILE
  if (-not (Test-Path -Path $profileDir)) {
    New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
  }
  if (-not (Test-Path -Path $PROFILE)) {
    New-Item -ItemType File -Path $PROFILE -Force | Out-Null
  }
  $profileContent = Get-Content -Path $PROFILE -Raw -ErrorAction SilentlyContinue
  $hookLine = "(& `"$targetPath`" shellenv powershell | Out-String) | Invoke-Expression"
  if ($profileContent -notlike "*$targetPath*shellenv*") {
    Add-Content -Path $PROFILE -Value "`n# HyperScoop Environment`n$hookLine"
    Write-Host "Added HyperScoop environment to PowerShell Profile: $PROFILE" -ForegroundColor Cyan
  }
} catch {
  Write-Verbose "Could not automatically update PowerShell profile: $_"
}

Write-Host "`nHyperScoop ($version) installed successfully! Run 'hp -h' to get started." -ForegroundColor Green
