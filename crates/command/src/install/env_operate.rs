use crate::init_env::{get_old_scoop_dir, get_scoop_cfg_path, init_scoop_global, init_user_scoop};
use crate::install::InstallOptions;
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{ManifestObj, StringArrayOrString};
use crate::utils::system::set_user_env_var;
use anyhow::bail;
use crossterm::style::Stylize;
use std::path::Path;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
use winreg::RegKey;

pub fn handle_env_set(
    env_set: ManifestObj,
    manifest: InstallManifest,
    options: &Box<[InstallOptions]>,
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
    let injects_var = format!(
        r#"
      $app = "{app_name}" ;
      $version = "{app_version}" ;
      $cmd ="uninstall" ;
      $global = $false  ;
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

    if let serde_json::Value::Object(env_set) = env_set {
        for (key, env_value) in env_set {
            let mut env_value = env_value.to_string().trim().to_string();
            if env_value.is_empty() {
                continue;
            }
            if env_value.contains('/') {
                env_value = env_value.replace('/', r"\");
            }
            if env_value.contains(r"\\") {
                env_value = env_value.replace(r"\\", r"\");
            }
            let cmd = format!(
                r#"Set-ItemProperty -Path "HKCU:\Environment" -Name "{key}" -Value {env_value}"#
            );

            let output = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(&app_dir)
                .arg(&injects_var)
                .arg(cmd)
                .output()?;
            if !output.status.success() {
                let error_output = String::from_utf8_lossy(&output.stderr);
                bail!(
                    "powershell failed to remove environment variable: {}",
                    error_output
                );
            }

            println!(
                "{} {}",
                "Env set successfully for".to_string().dark_green().bold(),
                key.to_string().dark_cyan().bold(),
            );
        }
    }
    Ok(())
}

pub fn handle_env_add_path(
    env_add_path: StringArrayOrString,
    app_current_dir: String,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let app_current_dir = app_current_dir.replace('/', r"\");
    if let StringArrayOrString::StringArray(paths) = env_add_path {
        for path in paths {
            add_bin_to_path(
                path.as_ref(),
                &app_current_dir,
                options,
            )?;
        }
    } else if let StringArrayOrString::String(path) = env_add_path {
        add_bin_to_path(path.as_ref(), &app_current_dir, options )?;
    }

    Ok(())
}

pub fn add_bin_to_path(
    path: &str,
    app_current_dir: &str,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let path = if path.eq(".") {
        Path::new(app_current_dir).to_str().unwrap().to_string()
    } else {
        Path::new(app_current_dir)
            .join(path)
            .to_str()
            .unwrap()
            .to_string()
    };
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey("Environment")?;
    let user_path: String = environment_key.get_value("PATH")?; 
    let  user_path_split: Vec<&str> = user_path.split(";").collect();
    if  user_path_split.contains(&path.as_str()) {
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
    let script =
        format!(r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "Machine")"#);
    if options.contains(&InstallOptions::Global) {
        let output = std::process::Command::new("powershell")
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
