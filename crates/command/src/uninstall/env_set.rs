#[cfg(windows)]
use crate::config::get_all_config;
#[cfg(windows)]
use crate::init_env::{get_old_scoop_dir, get_scoop_cfg_path, init_scoop_global, init_user_scoop};
#[cfg(windows)]
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crate::manifest::uninstall_manifest::UninstallManifest;
#[cfg(windows)]
use crate::utils::system::{
    delete_env_var, delete_global_env_var, set_global_env_var, set_user_env_var,
};
#[cfg(windows)]
use anyhow::bail;
use std::path::PathBuf;
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

pub fn env_var_rm(manifest: &UninstallManifest, is_global: bool) -> Result<(), anyhow::Error> {
    #[cfg(not(windows))]
    {
        let _ = (manifest, is_global);
        return Ok(());
    }
    #[cfg(windows)]
    {
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
                let root_key = if is_global {
                    RegKey::predef(HKEY_LOCAL_MACHINE)
                } else {
                    RegKey::predef(HKEY_CURRENT_USER)
                };
                let environment_key = if is_global {
                    root_key.open_subkey(
                        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                    )
                } else {
                    root_key.open_subkey("Environment")
                };
                let env_value: String = environment_key
                    .ok()
                    .and_then(|k| k.get_value(&key).ok())
                    .unwrap_or_default();
                if env_value.is_empty() {
                    continue;
                }
                let result = if is_global {
                    delete_global_env_var(key.as_str())
                } else {
                    delete_env_var(key.as_str())
                };

                if result.is_err() {
                    let cmd = if is_global {
                        format!(
                            r#"Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" -Name "{key}""#
                        )
                    } else {
                        format!(r#"Remove-ItemProperty -Path "HKCU:\Environment" -Name "{key}""#)
                    };
                    let output = std::process::Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg(cmd)
                        .output()?;
                    if !output.status.success() {
                        bail!("powershell failed to remove environment variable: {}", key);
                    }
                }
                log::debug!("env removed : key {}  ,value {}", key, env_value);
            }
        }
        Ok(())
    }
}

pub fn env_path_var_rm(
    current: &PathBuf,
    manifest: &UninstallManifest,
    is_global: bool,
) -> Result<(), anyhow::Error> {
    #[cfg(not(windows))]
    {
        let _ = (current, manifest, is_global);
        return Ok(());
    }
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::*;

        let paths_to_remove: Vec<PathBuf> = match manifest.env_add_path.clone() {
            Some(StringArrayOrString::String(env_add_path_str)) => {
                vec![if env_add_path_str == "." {
                    current.clone()
                } else {
                    current.join(env_add_path_str)
                }]
            }
            Some(StringArrayOrString::StringArray(env_add_path_arr)) => env_add_path_arr
                .into_iter()
                .map(|env_add_path_str| {
                    if env_add_path_str == "." {
                        current.clone()
                    } else {
                        current.join(env_add_path_str)
                    }
                })
                .collect(),
            _ => return Ok(()),
        };

        if paths_to_remove.is_empty() {
            return Ok(());
        }

        log::debug!("\n 要移除的路径变量: {:?}", paths_to_remove);
        let root_key = if is_global {
            RegKey::predef(HKEY_LOCAL_MACHINE)
        } else {
            RegKey::predef(HKEY_CURRENT_USER)
        };
        let environment_key = if is_global {
            root_key.open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")?
        } else {
            root_key.open_subkey("Environment")?
        };

        let user_path: String = environment_key.get_value("PATH")?;
        log::debug!("\n 当前用户的 PATH: {}", user_path);
        let mut paths: Vec<PathBuf> = std::env::split_paths(&user_path).collect();
        for p in &paths_to_remove {
            paths.retain(|item| item != p);
        }
        let new_user_path = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join(";");
        if user_path.eq(&new_user_path) {
            return Ok(());
        }
        log::debug!("\n 更新后的用户的 PATH: {}", new_user_path);

        let result = if is_global {
            set_global_env_var("Path", new_user_path.as_str())
        } else {
            set_user_env_var("Path", new_user_path.as_str())
        };
        if result.is_err() {
            eprintln!("Failed to remove path var by winreg, falling back to powershell");
            let target_scope = if is_global { "Machine" } else { "User" };
            let script = format!(
                r#"[System.Environment]::SetEnvironmentVariable("PATH", "{new_user_path}", "{target_scope}")"#
            );
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
        Ok(())
    }
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
