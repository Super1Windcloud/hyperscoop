use crate::config::get_all_config;
use crate::init_env::{
    get_old_scoop_dir, get_scoop_cfg_path, init_env_path, init_scoop_global_path,
};
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crate::manifest::uninstall_manifest::UninstallManifest;
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;

// 钩子类型枚举
#[derive(Debug, Clone)]
pub enum HookType {
    Uninstaller,
    PreUninstall,
    PostUninstall,
    PsModule,
}

// 处理器架构枚举
#[derive(Debug, Clone)]
pub enum ProcessorArchitecture {
    Bit32,
    Bit64,
    Arm64,
}

fn arch_specific(hook_type: HookType, manifest: &UninstallManifest ) -> Option<String> {
    match hook_type {
        HookType::Uninstaller => {
            let uninstaller = manifest.clone().uninstaller;
            if uninstaller.is_none() {
                return None;
            }
            let uninstaller = uninstaller.unwrap();
            let script = uninstaller.get("script").unwrap_or(&Value::Null);
            let result = match script {
                Value::String(s) => Some(s.clone()),
                Value::Array(arr) => {
                    let mut result = String::new();
                    for item in arr {
                        if let Value::String(s) = item {
                            result += s.as_str();
                            result.push('\n');
                        }
                    }
                    Some(result)
                }
                _ => None,
            };
            if result.is_some() {
                return Some(result.unwrap().to_string());
            }
            None
        }
        HookType::PreUninstall => {
            let pre_uninstall = manifest.clone().pre_uninstall;
            if pre_uninstall.is_none() {
                return None;
            }
            let pre_uninstall = pre_uninstall.unwrap();
            let result = match pre_uninstall {
                StringArrayOrString::String(s) => Some(s),
                StringArrayOrString::StringArray(arr) => {
                    let mut result = String::new();
                    for item in arr {
                        result += item.as_str();
                        result.push('\n');
                    }
                    Some(result)
                }
                _ => None,
            };
            if result.is_some() {
                return Some(result.unwrap());
            }
            None
        }
        HookType::PostUninstall => {
            let post_uninstall = manifest.clone().post_uninstall;
            if post_uninstall.is_none() {
                return None;
            }
            let post_uninstall = post_uninstall.unwrap();
            let result = match post_uninstall {
                StringArrayOrString::String(s) => Some(s),
                StringArrayOrString::StringArray(arr) => {
                    let mut result = String::new();
                    for item in arr {
                        result += item.as_str();
                        result.push('\n');
                    }
                    Some(result)
                }
                _ => None,
            };

            if result.is_some() {
                return Some(result.unwrap());
            }
            None
        }
        HookType::PsModule => None,
    }
}

// 模拟的 Invoke-HookScript 函数
pub fn invoke_hook_script(
    hook_type: HookType,
    manifest: &UninstallManifest,
    arch: &str,
) -> io::Result<()> {
    let script = arch_specific(hook_type.clone(), manifest );
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
        "$json =  '{}'; $manifest = $json | ConvertFrom-Json; $obj | ConvertTo-Json -Depth 10",
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
      $architecture = "{arch}" ;
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

    if let Some(script) = script {
        // 输出提示信息
        print!("Running {:?} script...", hook_type);
        log::info!("{}", script);
        io::stdout().flush()?;

        let output = Command::new("powershell")
            .arg("-Command")
            .arg(cfg_obj)
            .arg(manifest_obj)
            .arg(app_dir)
            .arg(injects_var)
            .arg(&script)
            .output()?;

        if output.status.success() {
            print!("\n{}", String::from_utf8(output.stdout).unwrap());
        } else {
            eprintln!("failed.");
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_hyperscoop;
    use rayon::prelude::*;
    use std::fs;
    #[test]
    fn test_invoke_uninstall_hook_script() {
        let hp = init_hyperscoop().unwrap();

        let bucket_path = hp.get_bucket_path();
        std::fs::read_dir(bucket_path)
            .unwrap()
            .par_bridge()
            .for_each(|entry| {
                let entry = entry.unwrap();
                let bucket = entry.path().join("bucket");

                fs::read_dir(bucket)
                    .unwrap()
                    .par_bridge()
                    .for_each(|entry| {
                        let entry = entry.unwrap();
                        let path = entry.path();

                        if path.is_file()
                            && path.extension().unwrap_or(std::ffi::OsStr::new("")) == "json"
                        {
                            let contents = fs::read_to_string(&path).unwrap_or_default();
                            let manifest: serde_json::Value =
                                serde_json::from_str(&contents).unwrap_or_default();
                            let post_uninstall = manifest["post_uninstall"].as_array();
                            let pre_uninstall = manifest["pre_uninstall"].as_array();
                            if post_uninstall.is_some() {
                                dbg!("{:#?}", post_uninstall.unwrap());
                            }
                            if pre_uninstall.is_some() {
                                dbg!("{:#?}", pre_uninstall.unwrap());
                                std::process::exit(0);
                            }
                        }
                    });
            });
    }
}
