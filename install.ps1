# 配置项
$repoOwner = "Super1Windcloud"
$repoName = "hyperscoop"
$exeName = "hp.exe"
$installDir = "$env:USERPROFILE\Tools\hyperscoop"
$headers = @{ "User-Agent" = "PowerShell" }

# 步骤 1：获取最新版本号
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

# 步骤 3：下载文件
$targetPath = Join-Path $installDir $exeName
try {
  Write-Host "下载中: $downloadUrl"
  Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath
  Write-Host "下载完成: $targetPath"
} catch {
  Write-Error "下载失败: $_"
  exit 1
}

# 步骤 4：将安装目录添加到用户 PATH 环境变量（如果尚未添加）
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

if ($currentPath -notlike "*$installDir*") {
  $newPath = "$currentPath;$installDir"
  try {
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "已添加到用户 PATH: $installDir"
    Write-Host "注意：重启 PowerShell 或重新登录后生效"
  } catch {
    Write-Error "添加 PATH 失败: $_"
    exit 1
  }
} else {
  Write-Host "路径已存在于用户 PATH 中: $installDir"
}
