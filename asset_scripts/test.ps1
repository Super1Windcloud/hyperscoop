function get_config($name, $default) {
  $name = $name.ToLowerInvariant()
  if($null -eq $scoopConfig.$name -and $null -ne $default) {
    return $default
  }
  return $scoopConfig.$name
}
function load_cfg($file) {
  if(!(Test-Path $file)) {
    return $null
  }
  try {
    $content = [System.IO.File]::ReadAllLines($file)
    return ($content | ConvertFrom-Json -ErrorAction Stop)
  } catch {
    Write-Host "ERROR loading $file`: $($_.exception.message)"
  }
}

function setup_proxy() {
  # note: '@' and ':' in password must be escaped, e.g. 'p@ssword' -> p\@ssword'
  $proxy = get_config PROXY
  if(!$proxy) {
    return
  }
  try {
    $credentials, $address = $proxy -split '(?<!\\)@'
    if(!$address) {
      $address, $credentials = $credentials, $null # no credentials supplied
    }

    if($address -eq 'none') {
      [net.webrequest]::defaultwebproxy = $null
    } elseif($address -ne 'default') {
      [net.webrequest]::defaultwebproxy = new-object net.webproxy "http://$address"
    }

    if($credentials -eq 'currentuser') {
      [net.webrequest]::defaultwebproxy.credentials = [net.credentialcache]::defaultcredentials
    } elseif($credentials) {
      $username, $password = $credentials -split '(?<!\\):' | ForEach-Object { $_ -replace '\\([@:])','$1' }
      [net.webrequest]::defaultwebproxy.credentials = new-object net.networkcredential($username, $password)
    }
  } catch {
    warn "Failed to use proxy '$proxy': $($_.exception.message)"
  }
}


$configHome = $env:XDG_CONFIG_HOME, "$env:USERPROFILE\.config" | Select-Object -First 1
$configFile = "$configHome\scoop\config.json"
$scoopConfig = load_cfg $configFile

setup_proxy
Write-Host "config: $scoopConfig"
$proxy = get_config PROXY

write-host "proxy: $proxy"

$options = @()

if ($proxy -ne 'none') {
  if ([Net.Webrequest]::DefaultWebProxy.Address) {
    write-host "using proxy: $([Net.Webrequest]::DefaultWebProxy.Address)"
    $options += "--all-proxy='$([Net.Webrequest]::DefaultWebProxy.Address.Authority)'"
  }
  if ([Net.Webrequest]::DefaultWebProxy.Credentials.UserName) {
    $options += "--all-proxy-user='$([Net.Webrequest]::DefaultWebProxy.Credentials.UserName)'"
  }
  if ([Net.Webrequest]::DefaultWebProxy.Credentials.Password) {
    $options += "--all-proxy-passwd='$([Net.Webrequest]::DefaultWebProxy.Credentials.Password)'"
  }
}

write-host "options: $options"
