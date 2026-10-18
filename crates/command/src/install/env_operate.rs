use crate::init_env::{get_old_scoop_dir, get_scoop_cfg_path, init_scoop_global, init_user_scoop};
use crate::install::InstallOptions;
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{ManifestObj, StringArrayOrString};
#[cfg(windows)]
use crate::utils::system::{set_global_env_var, set_user_env_var};
use anyhow::bail;
use crossterm::style::Stylize;
#[cfg(windows)]
use std::path::Path;
use std::process::Command;
#[cfg(windows)]
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
#[cfg(windows)]
use winreg::RegKey;

pub fn handle_env_set(
    env_set: ManifestObj,
    manifest: InstallManifest,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let app_name = manifest.name.unwrap_or(String::new());
    let app_version = manifest.version.unwrap_or(String::new());
    let scoop_home = if options.contains(&InstallOptions::Global) {
        init_scoop_global()
    } else {
        init_user_scoop()
    };
    let global_scoop_home = init_scoop_global();

    let app_dir = format!(
        r#"function app_dir($other_app) {{
      return  "{scoop_home}\apps\$other_app\current" ;
  }}"#
    );
    let old_scoop_dir = get_old_scoop_dir();
    let cfg_path = get_scoop_cfg_path();
    let is_global = options.contains(&InstallOptions::Global);
    let global_val = if is_global { "$true" } else { "$false" };
    let injects_var = format!(
        r#"
      $app = "{app_name}" ;
      $version = "{app_version}" ;
      $cmd ="install" ;
      $global = {global_val}  ;
      $scoopdir ="{scoop_home}" ;
      $dir = "{scoop_home}\apps\$app\current" ;
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

    let commands = env_set_commands(env_set, is_global);
    if commands.is_empty() {
        return Ok(());
    }
    let mut script = format!("$ErrorActionPreference = 'Stop';\n{app_dir}\n{injects_var}\n");
    for (_, command) in &commands {
        script.push_str(command);
        script.push_str(";\n");
    }
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()?;
    if !output.status.success() {
        bail!(
            "powershell failed to set environment variables: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for (key, _) in commands {
        println!(
            "{} '{}'",
            "Env set successfully for".dark_green().bold(),
            key.dark_cyan().bold()
        );
    }

    Ok(())
}

fn env_set_commands(env_set: ManifestObj, is_global: bool) -> Vec<(String, String)> {
    let serde_json::Value::Object(env_set) = env_set else {
        return Vec::new();
    };
    let registry_path = if is_global {
        r"HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    } else {
        r"HKCU:\Environment"
    };
    env_set
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value
                .to_string()
                .trim()
                .replace('/', r"\")
                .replace(r"\\", r"\");
            if value.is_empty() {
                return None;
            }
            // Keep PowerShell interpolation and expression handling from the original command.
            let escaped_key = key.replace('\'', "''");
            let command = format!(
                r#"Set-ItemProperty -Path '{registry_path}' -Name '{escaped_key}' -Value "{value}""#
            );
            Some((key, command))
        })
        .collect()
}

pub fn handle_env_add_path(
    env_add_path: StringArrayOrString,
    app_current_dir: String,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let app_current_dir = app_current_dir.replace('/', r"\");
    if let StringArrayOrString::StringArray(paths) = env_add_path {
        for path in paths {
            add_bin_to_path(path.as_ref(), &app_current_dir, options)?;
        }
    } else if let StringArrayOrString::String(path) = env_add_path {
        add_bin_to_path(path.as_ref(), &app_current_dir, options)?;
    }

    Ok(())
}

pub fn add_bin_to_path(
    path: &str,
    app_current_dir: &str,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    #[cfg(not(windows))]
    {
        let _ = (path, app_current_dir, options);
        return Ok(());
    }
    #[cfg(windows)]
    {
        let path = if path.eq(".") {
            Path::new(app_current_dir).to_str().unwrap().to_string()
        } else {
            Path::new(app_current_dir)
                .join(path)
                .to_str()
                .unwrap()
                .to_string()
        };
        let environment_key = if options.contains(&InstallOptions::Global) {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")?
        } else {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            hkcu.open_subkey("Environment")?
        };

        let user_path: String = environment_key.get_value("PATH")?;
        let user_path_split: Vec<&str> = user_path.split(";").collect();

        if user_path_split.contains(&path.as_str()) {
            log::warn!(
                "{} {} {}",
                "The path already exists in the user's PATH"
                    .to_string()
                    .dark_yellow()
                    .bold(),
                path.to_string().dark_yellow().bold(),
                "Skipping...".to_string().dark_yellow().bold()
            );
            return Ok(());
        }
        let user_path = if user_path.ends_with(";") {
            format!("{user_path}{path}")
        } else {
            format!("{user_path};{path}")
        };

        log::debug!("\n 更新后的用户的 PATH: {}", user_path);
        let script = format!(
            r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "Machine")"#
        );

        if options.contains(&InstallOptions::Global) {
            if set_global_env_var("Path", &user_path).is_ok() {
                return Ok(());
            }
            let output = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(script)
                .output()?;
            if !output.status.success() {
                bail!("Failed to remove path var");
            }
            Ok(())
        } else {
            set_user_env_var("Path", &user_path)?;
            Ok(())
        }
    }
}

mod test_env_operate {

    #[test] // 默认要执行很多次
    fn test_env_set() {
        use crate::install::{InstallOptions, handle_env_set};
        use crate::manifest::install_manifest::InstallManifest;
        let global = true;
        let path = r"A:\Scoop\buckets\main\bucket\yazi.json";

        let content = std::fs::read_to_string(path).unwrap();
        let mut manifest: InstallManifest = serde_json::from_str(&content).unwrap();
        let options = if global {
            vec![InstallOptions::Global]
        } else {
            vec![]
        };
        manifest.set_name("yazi");
        let env_set = manifest.env_set.clone().unwrap();
        handle_env_set(env_set, manifest, options.into_boxed_slice().as_ref()).unwrap();
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn batched_env_commands_preserve_scope_and_powershell_values() {
        let env = serde_json::json!({"TOOL_HOME": "$dir/bin", "TOOL_VERSION": "$version"});
        let commands = env_set_commands(env.clone(), false);
        assert_eq!(commands.len(), 2);
        assert!(commands[0].1.contains(r"HKCU:\Environment"));
        assert!(commands[0].1.contains(r"$dir\bin"));
        assert!(commands[1].1.contains("$version"));
        assert!(env_set_commands(env, true)[0].1.contains(r"HKLM:\SYSTEM"));
        assert!(env_set_commands(serde_json::json!({}), false).is_empty());
    }
}
