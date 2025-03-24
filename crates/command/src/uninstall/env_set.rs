use crate::config::get_all_config;
use crate::init_env::{
    get_old_scoop_dir, get_scoop_cfg_path, init_env_path, init_scoop_global_path,
};
use crate::manifest::uninstall_manifest::UninstallManifest;
use anyhow::bail;
use winreg::enums::*;
use winreg::RegKey;

pub fn env_var_rm(manifest: &UninstallManifest) -> Result<(), anyhow::Error> {
    let env_set = manifest.env_set.clone();
    if env_set.is_none() {
        return Ok(());
    }
    let env_set = env_set.unwrap();
    let app_name = manifest.name.clone().unwrap_or(String::new());
    let app_version = manifest.version.clone().unwrap_or(String::new());
    let cfg = get_all_config();
    let scoop_home = init_env_path();
    let global_scoop_home = init_scoop_global_path();
    let cfg = serde_json::to_string(&cfg).unwrap_or(String::new());
    let cfg_obj = format!(
        "$json =  '{}'; $cfg = $json | ConvertFrom-Json; $obj | ConvertTo-Json -Depth 10",
        cfg
    );
    let manifest_str = serde_json::to_string(&manifest).unwrap_or(String::new());
    let manifest_obj = format!(
        "$json = '{}' ; $manifest = $json | ConvertFrom-Json; $obj | ConvertTo-Json -Depth 10",
        manifest_str
    );
    let app_dir = format!(
        r#"function app_dir($other_app) {{
      return    "{scoop_home}\apps\$other_app\current" ;
  }}"#
    );
    let old_scoop_dir = get_old_scoop_dir();
    let cfg_path = get_scoop_cfg_path();
    let injects_var = format!(
        r#"
      $app = "{app_name}" ;
      $version = "{app_version}" ;
      $cmd ="uninstall" ;
      $global = $false  ;
      $scoopdir ="{scoop_home}" ;
      $dir = "{scoop_home}\apps\$app" ;
      $globaldir  ="{global_scoop_home}";
      $oldscoopdir  = "{old_scoop_dir}" ;
      $original_dir = "{scoop_home}\apps\$app\$version";
      $modulesdir  = "{scoop_home}\modules";
      $cachedir  =  "{scoop_home}\cache";
      $bucketsdir  = "{scoop_home}\buckets";
      $persist_dir  = "{scoop_home}\persist\$app";
      $cfgpath   ="{cfg_path}" ;
  "#
    );

    if let serde_json::Value::Object(env_set) = env_set {
        for (key, _) in env_set {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let environment_key = hkcu.open_subkey("Environment")?;
            let env_value: String = environment_key.get_value(&key).unwrap_or("".into());
            if env_value.is_empty() {
                continue;
            }
            let cmd = format!(r#"Remove-ItemProperty -Path "HKCU:\Environment" -Name {key}"#);

            let rm_env_var_pointer_path = format!(
                r#"
             if (Test-Path -Path  {env_value}  -PathType Container) {{
            Remove-Item -Path  {env_value} -Recurse -Force
            Write-Host "目录已删除:  {env_value}
              }} else {{
            Write-Host "目录不存在:  {env_value}
                  }}
            "#
            );

            let output = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(&cfg_obj)
                .arg(&manifest_obj)
                .arg(&app_dir)
                .arg(&injects_var)
                .arg(cmd)
                .arg(rm_env_var_pointer_path)
                .output()?;
            if !output.status.success() {
                bail!("powershell failed to set environment variable");
            }

            log::trace!("env set  : key {}  ,value {}", key, env_value);
        }
    }
    Ok(())
}



mod test { 
  #[allow(unused_imports )]
    use super::*;
    #[test]
    fn test_rm_env() {
        let mut manifest = UninstallManifest::new(r"A:\Scoop\buckets\DoveBoyApps\bucket\nvm.json");
         &manifest.set_name(&"nvm".to_string()) ;
        env_var_rm(&manifest).unwrap();
    }
}
