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
  exit 1
}

switch ($env:PROCESSOR_ARCHITECTURE) {
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
  Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath
  Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath
  $expectedHash = ((Get-Content $checksumPath -Raw) -split '\s+')[0].Trim().ToLowerInvariant()
  $actualHash = (Get-FileHash -Path $targetPath -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actualHash -ne $expectedHash) {
    Remove-Item -Path $targetPath -Force -ErrorAction SilentlyContinue
    Write-Error "Checksum verification failed. Expected $expectedHash, got $actualHash"
    exit 1
  }
  Write-Host "Download complete: $targetPath"
} catch {
  Write-Error "Download failed: $_"
  exit 1
}

$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

if ($currentPath -notlike "*$installDir*") {
  $newPath = "$currentPath;$installDir"
  try {
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "add  user PATH: $installDir"
  } catch {
    Write-Error "add user PATH failed: $_"
    exit 1
  }
} else {
  Write-Host "user PATH already contains: $installDir"
}


#  创建启动前安装所有依赖的程序
