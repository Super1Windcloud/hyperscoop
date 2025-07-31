$repoOwner = "Super1Windcloud"
$repoName = "hyperscoop"
$exeName = "hp.exe"
$installDir = "$env:USERPROFILE\Tools\hyperscoop"
$headers = @{ "User-Agent" = "PowerShell" }

try {
  $apiUrl = "https://api.github.com/repos/$repoOwner/$repoName/releases/latest"
  $response = Invoke-RestMethod -Uri $apiUrl -Headers $headers
  $version = $response.tag_name
  Write-Host "最新版本: $version"
} catch {
  Write-Error "获取版本失败: $_"
  exit 1
}

$downloadUrl = "https://github.com/$repoOwner/$repoName/releases/download/$version/$exeName"

if (-not (Test-Path -Path $installDir)) {
  New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$targetPath = Join-Path $installDir $exeName
try {
  Write-Host "下载中: $downloadUrl"
  Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath
  Write-Host "下载完成: $targetPath"
} catch {
  Write-Error "下载失败: $_"
  exit 1
}

$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

if ($currentPath -notlike "*$installDir*") {
  $newPath = "$currentPath;$installDir"
  try {
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "add  user PATH: $installDir"
  } catch {
    Write-Error "添加 PATH 失败: $_"
    exit 1
  }
} else {
  Write-Host "路径已存在于用户 PATH 中: $installDir"
}
