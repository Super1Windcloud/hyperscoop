use crate::check_self_update::auto_check_hp_update;
use crate::command_args::update::UpdateArgs;
use crate::Cli;
use anyhow::anyhow;
use clap::CommandFactory;
use command_util_lib::init_env::{get_app_current_dir, get_app_current_dir_global};
use command_util_lib::install::UpdateOptions::ForceUpdateOverride;
use command_util_lib::install::{
    install_and_replace_hp, InstallOptions, UpdateOptions,
};
use command_util_lib::update::*;
use command_util_lib::utils::utility::update_scoop_config_last_update_time;
use crossterm::style::Stylize;
use std::fs::File;
use std::io::Write;
use std::process::Command;

pub async fn execute_update_command(update_args: UpdateArgs) -> Result<(), anyhow::Error> {
    let options = inject_update_user_options(&update_args)?;
    if update_args.update_self_and_buckets {
        println!("{}", "开始更新hp和buckets".dark_cyan().bold());
        update_hp(&options).await?;
        update_buckets()?;
        return Ok(());
    }
    if update_args.all {
        log::debug!("update all app ");
        update_all_apps(&options).await?;
        return Ok(());
    }
    if update_args.app_name.is_none() {
        return Ok(());
    }
    let app_name = update_args.app_name.unwrap();
    log::debug!("update app: {}", app_name);
    if app_name.to_lowercase() == "hp" {
        update_hp(&options).await?;
        return Ok(());
    }
    update_specific_app(&app_name, &options).await?;
    Ok(())
}

fn inject_update_user_options(args: &UpdateArgs) -> anyhow::Result<Vec<UpdateOptions>> {
    let mut options = vec![];

    if args.global {
        options.push(UpdateOptions::Global);
    }
    if args.update_self_and_buckets {
        options.push(UpdateOptions::UpdateHpAndBuckets);
    }
    if args.all {
        options.push(UpdateOptions::UpdateAllAPP);
    }
    if args.no_use_download_cache {
        options.push(UpdateOptions::NoUseDownloadCache);
    }
    if args.skip_hash_check {
        options.push(UpdateOptions::SkipDownloadHashCheck);
    }
    if args.remove_old_app {
        options.push(UpdateOptions::RemoveOldVersionApp);
    }
    if args.no_auto_download_dependencies {
        options.push(UpdateOptions::NoAutoDownloadDepends);
    }
    if args.force_update_override {
        options.push(ForceUpdateOverride);
    }
    Ok(options)
}

pub(crate) fn update_buckets() -> Result<(), anyhow::Error> {
    update_scoop_bar()?;
    let status = check_bucket_update_status()?;
    if !status {
        return Ok(());
    }
    update_all_buckets_bar()?;
    update_scoop_config_last_update_time();
    Ok(())
}

pub async fn update_hp(options: &[UpdateOptions]) -> Result<(), anyhow::Error> {
    let cmd = Cli::command();
    let version = cmd.get_version().ok_or(anyhow!("hp version is empty"))?;
    let result = if !options.contains(&ForceUpdateOverride) {
        auto_check_hp_update().await?
    } else {
        true
    };
    if !result {
        println!(
            "{}",
            format!("hp '{version}' are up to date")
                .to_string()
                .dark_green()
                .bold()
        );
        return Ok(());
    }
    let options = transform_update_options_to_install(options);
    let global = if options.contains(&InstallOptions::Global) {
        true
    } else {
        false
    };
    install_and_replace_hp(options.as_slice()).await?;
    launch_update_script(global).expect("update hp script failed");
    std::process::exit(0);
}

fn launch_update_script(global: bool) -> anyhow::Result<()> {
    let script_content = r#"@echo off
setlocal enabledelayedexpansion
timeout /t 2 > nul
:waitloop
tasklist | findstr /i "hp.exe" > nul
if not errorlevel 1 (
    timeout /t 1 > nul
    goto waitloop
)

move /y "%~dp0hp_updater.exe" "%~dp0hp.exe" > nul

for /f "delims=" %%v in ('"%~dp0hp.exe" -V') do (
    set VERSION=%%v
)

powershell -Command "Write-Host 'Hp Updates Successfully! Current Version ：!VERSION!' -ForegroundColor Green"

endlocal
"#;
    let hp_current = if global {
        get_app_current_dir_global("hp")
    } else {
        get_app_current_dir("hp")
    };
    let  updater = format!("{hp_current}\\updater.bat"); 
    let mut file = File::create(&updater)?;
    file.write_all(script_content.as_bytes())?;

    // Command::new("cmd")
    //     .args(&["/C",  &updater])
    //     .spawn()?; // 非阻塞方式启动

    Ok(())
}
