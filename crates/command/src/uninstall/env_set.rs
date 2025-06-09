use crate::config::get_all_config;
use crate::init_env::{get_old_scoop_dir, get_scoop_cfg_path, init_scoop_global, init_user_scoop};
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crate::manifest::uninstall_manifest::UninstallManifest;
use crate::utils::system::{
    delete_env_var, delete_global_env_var, set_global_env_var, set_user_env_var,
};
use anyhow::bail;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

pub fn env_var_rm(manifest: &UninstallManifest, is_global: bool) -> Result<(), anyhow::Error> {
    let env_set = manifest.env_set.clone();
    if env_set.is_none() {
        return Ok(());
    }
    let env_set = env_set.unwrap();
    let app_name = manifest.name.clone().unwrap_or(String::new());
    let app_version = manifest.version.clone().unwrap_or(String::new());
    let cfg = get_all_config();
    let scoop_home = init_user_scoop();
    let global_scoop_home = init_scoop_global();
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
            let environment_key = if is_global {
                hkcu.open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")?
            } else {
                hkcu.open_subkey("Environment")?
            };
            let env_value: String = environment_key.get_value(&key).unwrap_or("".into());
            if env_value.is_empty() {
                continue;
            }
            let _cmd = if is_global {
                delete_global_env_var(key.as_str())?;
                format!(
                    r#"Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" -Name {key}"#
                )
            } else {
                delete_env_var(key.as_str())?;
                format!(r#"Remove-ItemProperty -Path "HKCU:\Environment" -Name {key}"#)
            };

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
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&cfg_obj)
                .arg(&manifest_obj)
                .arg(&app_dir)
                .arg(&injects_var)
                .arg(rm_env_var_pointer_path)
                .output()?;

            if !output.status.success() {
                bail!("powershell failed to set environment variable");
            }
            log::debug!("env removed : key {}  ,value {}", key, env_value);
        }
    }
    Ok(())
}

pub fn env_path_var_rm(
    current: &PathBuf,
    manifest: &UninstallManifest,
    is_global: bool,
) -> Result<(), anyhow::Error> {
    use winreg::enums::*;
    use winreg::RegKey;
    if let Some(StringArrayOrString::String(env_add_path_str)) = manifest.env_add_path.clone() {
        let path_var = if env_add_path_str == "." {
            current.clone()
        } else {
            current.join(env_add_path_str)
        }; 
        log::debug!("\n 要移除的路径变量: {}", path_var.to_string_lossy()); 
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let environment_key = if is_global {
            hkcu.open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")?
        } else {
            hkcu.open_subkey("Environment")?
        };

        let user_path: String = environment_key.get_value("PATH")?;
        log::debug!("\n 当前用户的 PATH: {}", user_path);
        let mut paths: Vec<PathBuf> = std::env::split_paths(&user_path).collect();
        paths.retain(|p| p != &path_var);
        let new_user_path = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join(";");
        if user_path.eq(new_user_path.as_str()) {
            return Ok(());
        }
        log::debug!("\n 更新后的用户的 PATH: {}", new_user_path);

        let script = if is_global {
            format!(
                r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "Machine")"#
            )
        } else {
            format!(r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "User")"#)
        };
        let result = if is_global {
            set_global_env_var("Path", new_user_path.as_str())
        } else {
            set_user_env_var("Path", new_user_path.as_str())
        };
        if result.is_err() {
            eprintln!("Failed to remove path var by winreg");
            let output = std::process::Command::new("powershell")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg(script)
                .output()?;
            if !output.status.success() {
                bail!("Failed to remove path var");
            }
        }
    } else if let Some(StringArrayOrString::StringArray(env_add_path_arr)) =
        manifest.env_add_path.clone()
    {
        let env_add_path_arr = env_add_path_arr
            .iter()
            .map(|env_add_path_str| {
                if env_add_path_str == "." {
                    current.clone()
                } else {
                    current.join(env_add_path_str)
                }
            })
            .collect::<Vec<PathBuf>>();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let environment_key = hkcu.open_subkey("Environment")?;

        let user_path: String = environment_key.get_value("PATH")?;
        let origin = user_path.clone();
        log::debug!("\n 当前用户的 PATH: {}", user_path);
        let mut paths: Vec<PathBuf> = std::env::split_paths(&user_path).collect();

        for path_var in env_add_path_arr {
            paths.retain(|p| p != &path_var);
        }

        let user_path = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join(";");
        if user_path == origin {
            log::debug!("\n 没有需要移除的路径变量");
            return Ok(());
        }
         log::debug!("\n 更新后的用户的 PATH: {}", user_path);

        if is_global {
            set_global_env_var("Path", user_path.as_str())?;
        } else {
            set_user_env_var("Path", user_path.as_str())?;
        }
      
    }
    Ok(())
}

mod test {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_rm_env() {
        let mut manifest = UninstallManifest::new(r"A:\Scoop\buckets\DoveBoyApps\bucket\nvm.json");
        manifest = manifest.set_name(&"nvm".to_string()).to_owned();
        env_var_rm(&manifest, false).unwrap();
    }
}
