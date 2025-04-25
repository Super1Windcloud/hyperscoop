function Fibonacci($n) {
    if ($n -le 1) {
        return $n
    }
    return (Fibonacci($n - 1) + Fibonacci($n - 2))
}

$start_time = Get-Date
Fibonacci 30 | Out-Null
$end_time = Get-Date

$elapsed = $end_time - $start_time
Write-Output "PowerShell: $($elapsed.TotalSeconds) seconds"

$varialbFuck= 123 ;  $demo = "hello world"
$varialbfuck = 456 ;  $demo = "goodbye world"
write-host  $varialbFuck , $demo


$stats = git log --pretty=tformat: --numstat |
  Where-Object { $_ -match "^\d" } |
  ForEach-Object {
    $parts = $_ -split "\s+"
    [PSCustomObject]@{
      Added   = [int]$parts[0]
      Deleted = [int]$parts[1]
    }
  }

$added = ($stats | Measure-Object -Property Added -Sum).Sum
$deleted = ($stats | Measure-Object -Property Deleted -Sum).Sum
$net = $added - $deleted

Write-Host "Added lines: $added"
Write-Host "Deleted lines: $deleted"
Write-Host "Total (net): $net"

