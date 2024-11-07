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
