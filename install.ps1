# Set TLS 1.2 protocol for older PowerShell versions
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$repoOwner = "Super1Windcloud"
$repoName = "hyperscoop"
$installDir = "$env:USERPROFILE\Tools\hyperscoop"
$headers = @{ "User-Agent" = "PowerShell" }

try {
  $apiUrl = "https://api.github.com/repos/$repoOwner/$repoName/releases/latest"
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
$downloadUrl = "https://github.com/$repoOwner/$repoName/releases/download/$version/$assetName"
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

Write-Host "HyperScoop ($version) installed successfully! Run 'hp -h' to get started." -ForegroundColor Green
