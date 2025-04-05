try {
  $targetPath = "A:\Webstorm\Tauri_electron_app\electron_vite_react\HyperSend"

  if (Test-Path $targetPath) {
    $currentDirectoryPath = Get-Location
    Set-Location -Path $targetPath
    pnpm dev
    Set-Location -Path $currentDirectoryPath
  } else {
    Write-Error "目标路径不存在：$targetPath"
  }
} catch {
  Write-Error "执行过程中发生错误：$_"
}


