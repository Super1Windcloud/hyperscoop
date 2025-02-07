$targetPath = "C:\Program Files\MyApp\bin"
$scope = "Scoop"

if (-not (($env:Path -split ';') -contains $targetPath)) {
  $newPath = [Environment]::GetEnvironmentVariable("Path", $scope)
  $newPath += ";$targetPath"
  [Environment]::SetEnvironmentVariable("Path", $newPath, $scope)

  # 通知系统更新（仅Windows有效）
  Send-EnvUpdateSignal
}
