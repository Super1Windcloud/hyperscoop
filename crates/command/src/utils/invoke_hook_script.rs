use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;

// 钩子类型枚举
#[derive(Debug, Clone, Copy)]
pub enum HookType {
    Installer,
    PreInstall,
    PostInstall,
    Uninstaller,
    PreUninstall,
    PostUninstall,
    PsModule,
}

// 处理器架构枚举
#[derive(Debug, Clone, Copy)]
pub enum ProcessorArchitecture {
    Bit32,
    Bit64,
    Arm64,
}

// 模拟的清单结构
struct Manifest {
    // 假设清单包含钩子脚本
    installer_script: Option<String>,
    pre_install_script: Option<String>,
    post_install_script: Option<String>,
    uninstaller_script: Option<String>,
    pre_uninstall_script: Option<String>,
    post_uninstall_script: Option<String>,
}
#[allow(unused)]
fn arch_specific(hook_type: HookType, manifest: &Value, arch: &str) -> Option<String> {
    match hook_type {
        HookType::Installer => match manifest.get("installer")?.as_str() {
            None => manifest["installer"]
                .as_object()
                .map(|o| serde_json::to_string(o).unwrap()),
            Some(installer) => Some(String::from(installer)),
        },
        HookType::PreInstall => manifest["pre_install"].as_array().and_then(|arr| {
            let joined = arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Some(joined)
        }),
        HookType::PostInstall => manifest["post_install"].as_array().and_then(|arr| {
            let joined = arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Some(joined)
        }),
        HookType::Uninstaller => match manifest["uninstaller"].as_str() {
            None => manifest["uninstaller"]
                .as_object()
                .map(|o| serde_json::to_string(o).unwrap()),
            Some(uninstaller) => Some(String::from(uninstaller)),
        },
        HookType::PreUninstall => manifest["pre_uninstall"].as_array().and_then(|arr| {
            let joined = arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Some(joined)
        }),
        HookType::PostUninstall => manifest["post_uninstall"].as_array().and_then(|arr| {
            let joined = arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Some(joined)
        }),
       _ => None,
    }
}

// 模拟的 Invoke-HookScript 函数
pub fn invoke_hook_script(hook_type: HookType, manifest: &Value, arch: &str) -> io::Result<()> {
    // 获取钩子脚本
    let script = arch_specific(hook_type, manifest, arch);

    if let Some(script) = script {
        // 输出提示信息
        print!("Running {:?} script...", hook_type);
        log::info!("{}", script);
        io::stdout().flush()?;

        let output = Command::new("powershell")
            .arg("-Command")
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
