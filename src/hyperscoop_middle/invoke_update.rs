use crate::check_self_update::{auto_check_hp_update, get_app_old_version};
use crate::command_args::update::UpdateArgs;
use command_util_lib::init_env::{
    get_app_current_bin_path, get_app_current_dir, get_app_current_dir_global, get_app_dir,
    get_app_version_dir,
};
use command_util_lib::install::UpdateOptions::ForceUpdateOverride;
use command_util_lib::install::{install_and_replace_hp, InstallOptions, UpdateOptions};
use command_util_lib::update::*;
use command_util_lib::utils::utility::update_scoop_config_last_update_time;
use crossterm::style::Stylize;
use line_ending::LineEnding;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub async fn execute_update_command(update_args: UpdateArgs) -> Result<(), anyhow::Error> {
    let options = inject_update_user_options(&update_args)?;
    if update_args.update_self_and_buckets {
        println!("{}", "开始更新hp和buckets".dark_cyan().bold());
        update_buckets()?;
        update_hp(&options).await?;
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
    // update_scoop_bar().expect("update scoop bar failed");
    // let status = check_bucket_update_status()?;
    // if !status {
    //     return Ok(());
    // }
    update_all_buckets_bar().expect("update all buckets bar failed");
    update_scoop_config_last_update_time();
    Ok(())
}

pub async fn update_hp(options: &[UpdateOptions]) -> Result<(), anyhow::Error> {
    let update_options = options;
    let install_options = transform_update_options_to_install(options);
    let global = if install_options.contains(&InstallOptions::Global) {
        true
    } else {
        false
    };

    let old_version = if !update_options.contains(&ForceUpdateOverride) {
        let app_dir = get_app_dir("hp");
        if !Path::new(&app_dir).exists() {
            let version = install_and_replace_hp(install_options.as_slice())
                .await
                .expect("hp update failed");
            launch_update_script(global, "", false ).expect("update hp script failed");
            println!(
                "{}",
                format!("Hp Latest Version('{version}') Update Successfully! ❤️‍🔥💝🐉🍾🎉")
                    .to_string()
                    .dark_green()
                    .bold()
            );
            return Ok(());
        }
        get_app_old_version("hp", update_options).expect("get app old version failed")
    } else {
        let old_version =
            get_app_old_version("hp", update_options).expect("get app old version failed");
        if old_version.is_empty() {
          let hp_exe = get_app_current_bin_path("hp", "hp.exe");
          let output = Command::new(hp_exe)
            .arg("--version")
            .output()
            .expect("failed to get hp version");
          let version = String::from_utf8(output.stdout).expect("failed to parse hp version");
          version.trim().to_string()
        } else {
            old_version
        }
    };
    let result = if !update_options.contains(&ForceUpdateOverride) {
        auto_check_hp_update(Some(old_version.as_str())).await?
    } else {
        true
    };
    if !result {
        println!(
            "{}",
            format!("hp '{old_version}' are up to date")
                .to_string()
                .dark_green()
                .bold()
        );
        return Ok(());
    }
    let app_old_version_dir = get_app_version_dir("hp", old_version.as_str());

    let version = install_and_replace_hp(install_options.as_slice())
        .await
        .expect("install_and_replace_hp  failed");
    
    if   update_options.contains(&ForceUpdateOverride) {
         launch_update_script(global, "" , true).expect("update hp script failed");
    } else {
        launch_update_script(global, app_old_version_dir.as_str() ,  false ).expect("update hp script failed");
    } 
    println!(
        "{}",
        format!("Hp Latest Version('{version}') Update Successfully! ❤️‍🔥💝🐉🍾🎉")
            .to_string()
            .dark_green()
            .bold()
    );
    Ok(())
}

fn launch_update_script(global: bool, old_version_dir: &str, replace_old_hp   : bool) -> anyhow::Result<()> {
    let hp_current = if global {
        get_app_current_dir_global("hp")
    } else {
        get_app_current_dir("hp")
    };

    let script_content = format!(
        r#"@echo off
chcp 65001 > nul
setlocal enabledelayedexpansion
timeout /t 1 > nul
:waitloop
tasklist /fi "imagename eq hp.exe" | findstr /i "hp.exe" > nul
if not errorlevel 1 (
    timeout /t 1 > nul
    goto waitloop
)


if "{old_version_dir}" == "" (
    exit /b 0
)

if exist "{old_version_dir}" (
    rmdir  /S /Q "{old_version_dir}" 
    if exist "{old_version_dir}"  (
        echo ERROR: Directory still exists after deletion.
        exit /b 1
    )
)

rename  "{hp_current}\hp_updater.exe" "{hp_current}\hp.exe" > nul
if not exist "{hp_current}\hp.exe" (
    echo ERROR: Failed to rename hp.exe to hp.old.exe.
    exit /b 1
)
endlocal
"#
    );

  let  force_override_script =  format!(r#"@echo off
chcp 65001 > nul
setlocal enabledelayedexpansion
timeout /t 1 > nul
:waitloop
tasklist /fi "imagename eq hp.exe" | findstr /i "hp.exe" > nul
if not errorlevel 1 (
    timeout /t 1 > nul
    goto waitloop
)

cd /d "{hp_current}"

:: 删除旧的 hp.exe（如果存在）
if exist "hp.exe" (
    del /f /q "hp.exe" > nul
    if exist "hp.exe" (
        echo ERROR: Failed to delete hp.exe.
        exit /b 1
    )
)

:: 重命名 hp_updater.exe 为 hp.exe
if exist "hp_updater.exe" (
    rename "hp_updater.exe" "hp.exe"
) else (
    echo ERROR: hp_updater.exe 不存在！
    exit /b 1
)

:: 检查是否重命名成功
if not exist "hp.exe" (
    echo ERROR: Failed to rename hp_updater.exe to hp.exe.
    exit /b 1
)

endlocal
"#);

    let script_content = if  !replace_old_hp {
        script_content
    } else {
        force_override_script
    };  
    let updater = format!("{hp_current}\\updater.bat");
    let mut file = File::create(&updater).expect("Failed to create updater.bat");
    let script_content = script_content.replace(LineEnding::LF.as_str(), LineEnding::CRLF.as_str());
    file.write_all(script_content.as_bytes())?;
    Command::new("cmd").args(&["/C", &updater]).spawn()?;
    Ok(())
}
