use crate::command_args::install::InstallArgs;
use anyhow::bail;
use command_util_lib::install::*;
use std::path::Path;

pub async fn execute_install_command(args: InstallArgs) -> Result<(), anyhow::Error> {
    if args.app_name.is_none() {
        return Ok(());
    }
    let options = inject_user_options(&args)?;
    let app_name = args.app_name.unwrap();
    let app_name = convert_path(app_name.trim());
    if Path::new(&app_name).exists() {
        log::trace!("manifest file {}", app_name);
        let manifest_path = app_name;
        install_app_from_local_manifest_file(manifest_path, options).await?;
        return Ok(());
    }
    if contains_special_char(app_name.as_str()) {
        bail!("指定的APP格式错误 error char")
    }

    if app_name.contains("/") {
        if app_name.contains('@') {
            bail!("指定的App格式不正确")
        }
        let split_arg = app_name.split('/').collect::<Vec<&str>>();
        if split_arg.iter().count() == 2 {
            let bucket = split_arg[0];
            let app_name = split_arg[1];
            if bucket.is_empty() || app_name.is_empty() {
                bail!("指定的App格式不正确")
            }
            install_from_specific_bucket(bucket, app_name, &options ).await?;
            return Ok(());
        } else if split_arg.iter().count() > 2 || split_arg.len() == 1 {
            bail!("指定的APP格式错误")
        }
    }
    if app_name.contains('@') {
        let split_version = app_name.split('@').collect::<Vec<&str>>();
        if split_version.iter().count() == 2 {
            let app_name = split_version[0];
            let app_version = split_version[1];
            if app_name.is_empty() || app_version.is_empty() {
                bail!("指定的APP格式错误")
            }
            install_app_specific_version(app_name, app_version, &options ).await?;
            return Ok(());
        } else if split_version.len() == 1 || split_version.len() > 2 {
            bail!("指定的APP格式错误")
        }
    }
    if contains_special_char(app_name.as_str()) {
        bail!("指定的APP格式错误")
    }
    install_app(app_name.as_str() , &options ).await?;
    Ok(())
}

pub fn inject_user_options(install_args: &InstallArgs) -> anyhow::Result<Vec<InstallOptions>> {
    let mut install_options = vec![];
    if install_args.arch.is_some() { 
       let arch = install_args.arch.clone().unwrap();
        install_options.push(InstallOptions::ArchOptions(arch ));
    }
    if install_args.skip_download_hash_check {
        install_options.push(InstallOptions::SkipDownloadHashCheck)
    }
    if install_args.update_hp_and_buckets {
        install_options.push(InstallOptions::UpdateHpAndBuckets)
    }
    if install_args.no_use_download_cache {
        install_options.push(InstallOptions::NoUseDownloadCache)
    }
    if install_args.no_auto_download_dependencies {
        install_options.push(InstallOptions::NoAutoDownloadDepends)
    }

    Ok(install_options)
}

fn contains_special_char(s: &str) -> bool {
    let special_chars = r#"!#$%^&*()+=\[]\{}|;':",.<>?~`"#;
    s.chars().any(|c| special_chars.contains(c))
}

fn convert_path(path: &str) -> String {
    let path = path.replace("\\", "/");
    path
}
