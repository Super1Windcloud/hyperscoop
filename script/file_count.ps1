param (
    [Switch]$ShowFiles, # 新增 Flag：显示所有子文件的文件名
    [Switch]$ShowDirs   # 保留 Flag：仅显示目录路径
)

$root = "A:\"

if (!(Test-Path $root)) {
    Write-Host "错误: A 盘不存在或不可访问" -ForegroundColor Red
    exit
}

# 1. 获取目标目录 (github*)
$targetDirs = Get-ChildItem -Path $root -Directory -Filter "github*"

if ($targetDirs.Count -eq 0) {
    Write-Host "未找到以 github 开头的目录。" -ForegroundColor Yellow
    exit
}

# --- 逻辑执行 ---

$totalFileCount = 0

foreach ($dir in $targetDirs) {
    # 获取该目录下所有的文件
    $files = Get-ChildItem -Path $dir.FullName -File
    $totalFileCount += $files.Count

    # 如果启用了 -ShowDirs 或 -ShowFiles，先打印目录名作为层级
    if ($ShowDirs -or $ShowFiles) {
        Write-Host "`n目录: $($dir.FullName)" -ForegroundColor Cyan -Bold
    }

    # 如果启用了 -ShowFiles，循环打印该目录下所有文件名
    if ($ShowFiles) {
        if ($files.Count -gt 0) {
            foreach ($f in $files) {
                Write-Host "  |-- $($f.Name)" -ForegroundColor Gray
            }
        } else {
            Write-Host "  (空目录)" -ForegroundColor DarkGray
        }
    }
}

# 最终汇总统计
Write-Host "`n" + ("=" * 30)
Write-Host "总计文件数: $totalFileCount" -ForegroundColor Green
Write-Host ("=" * 30)
