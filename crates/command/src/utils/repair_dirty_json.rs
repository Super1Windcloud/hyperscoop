use regex::Regex;
use repair_json::{repair, Builder};


pub const DEMO_JSON: &str = r#"
{
    "version": "20141215",
    "license": "apache-2.0",
    "homepage": "http://itouhiro.hatenablog.com/entry/20140917/font",
    "url": "https://ja.osdn.net/frs/chamber_redir.php?m=iij&f=%2Fusers%2F7%2F75872FNasuFont-20141215.zip#/dl.zip",
    "hash": "bacdb09369d841cc0203f292602622828677136e7765eeb8709ba2286e33e3b0",
    "extract_dir": "NasuFont20141215",
    "installer": {
        "script": "
            Get-ChildItem $dir -filter '*.ttf' | ForEach-Object {
                New-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts' -Name $_.Name.Replace($_.Extension, ' (TrueType)') -Value $_.Name -Force | Out-Null
                Copy-Item \"$dir\\$_\" -destination \"$env:windir\\Fonts\"
            }
        "
    },
    "uninstaller": {
        "script": "
            Get-ChildItem $dir -filter '*.ttf' | ForEach-Object {
                Remove-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts' -Name $_.Name.Replace($_.Extension, ' (TrueType)') -Force -ErrorAction SilentlyContinue
                Remove-Item \"$env:windir\\Fonts\\$($_.Name)\" -Force -ErrorAction SilentlyContinue
            }
            Write-Host \"The 'Nasu' Font family has been uninstalled and will not be present after restarting your computer.\" -Foreground Magenta
        "
    }
}
"#;


pub fn fix_dirty_json(dirty_json: &str) -> Result<String, anyhow::Error> {
  let mut json_str = repairing_json(&dirty_json).unwrap();
  println!("{}", json_str);
  // 删除控制字符
  let re = Regex::new(r"[\x00-\x1F\x7F]")?; // 匹配控制字符
  json_str = re.replace_all(&json_str, "").to_string(); //
  let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

  return Ok(json_value.to_string());
}
fn repairing_json(dirty_json: &str) -> Result<String, anyhow::Error> {
  //  替换掉单引号
  let mut json_str = dirty_json.trim().to_string();
  //  json_str = json_str.replace("'", "\"");
  return Ok(json_str);
}
