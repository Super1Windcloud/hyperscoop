use crate::init_env::{
    get_scoop_config_path,
};
use anyhow::bail;
use crossterm::style::Stylize;
use std::process::Stdio;
use crate::install::install_from_specific_bucket;

pub fn write_into_scoop_config(config: String) {
    let default_config_path = get_scoop_config_path().unwrap();
    log::info!("{:?}", default_config_path);
    std::fs::write(default_config_path, config).unwrap();
}

pub fn add_buckets(buckets: Vec<(&str, &str)>, path: String) -> Result<(), anyhow::Error> {
    for bucket in buckets {
        if bucket.0.is_empty() || bucket.1.is_empty() {
            bail!(
                "Config_File Error: bucket name or bucket source is empty on {}",
                path.red().bold()
            )
        }
        let name = bucket.0;
        let source = bucket.1;
        log::info!("{:?}", name);
        log::info!("{:?}", source);
        invoke_hp_bucket_add(name, source);
    }

    Ok(())
}

fn invoke_hp_bucket_add(name: &str, url: &str) {
    let cmd = std::process::Command::new("hp")
        .arg("bucket")
        .arg("add")
        .arg(name)
        .arg(url)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("failed to invoke HP bucket add");
    if !cmd.status.success() {
        {
            let stderr = String::from_utf8_lossy(&cmd.stderr);
            eprintln!("Error: {}", stderr);
        }
    }
}

pub   fn install_apps(app_info: Vec<(&str, &str, &str)>, path: String) -> Result<(), anyhow::Error> {
    for (app_name, bucket, _ ) in app_info {
        if app_name.is_empty() || bucket.is_empty() {
            bail!(
                "Config_File Error: app name or bucket name is empty, on {}",
                path.red().bold()
            )
        }
        invoke_hp_install(app_name, bucket)?;
    }

    Ok(())
}

fn invoke_hp_install(app_name: &str, bucket: &str) -> Result<(), anyhow::Error> {
    install_from_specific_bucket(bucket, app_name, &*vec![])?;
    Ok(())
}


