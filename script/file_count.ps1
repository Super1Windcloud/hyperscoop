$root = "A:\";

if (!(Test-Path $root)) {
  Write-Host "A 盘不存在或不可访问"
  exit
}

$dirs = Get-ChildItem -Path $root -Directory -Filter "github*"

function Get-ChildDir-FileCount($dirs ) {
  $file = @()
  foreach ($dir in $dirs) {
    $file += Get-ChildItem -Path $dir.FullName -File | Measure-Object
  }
  $count = $file | Measure-Object -Property Count -Sum
  Write-Host  "total count :  $($count.Sum)"
  return $count
}
function Output-FileNames($dirs) {
  if ($dirs.Count -eq 0) {
    Write-Host "未找到以 github 开头的目录。"
  } else {
    Write-Host "找到以下以 github 开头的目录：`n"
    foreach ($dir in $dirs) {
      Write-Host $dir.FullName
    }
  }

}

Get-ChildDir-FileCount $dirs
