$repoOwner = "Super1Windcloud"
$repoName = "hyperscoop"
$exeName = "hp.exe"
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

$downloadUrl = "https://github.com/$repoOwner/$repoName/releases/download/$version/$exeName"

if (-not (Test-Path -Path $installDir)) {
  New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$targetPath = Join-Path $installDir $exeName
try {
  Write-Host "Downloading: $downloadUrl"
  Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath
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
