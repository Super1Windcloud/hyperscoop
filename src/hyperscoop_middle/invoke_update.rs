use crate::command_args::update::UpdateArgs;
use command_util_lib::install::UpdateOptions;
use command_util_lib::update::*;
use command_util_lib::utils::utility::update_scoop_config_last_update_time;
use crossterm::style::Stylize;

pub async fn execute_update_command(update_args: UpdateArgs) -> Result<(), anyhow::Error> {
    let options = inject_update_user_options(&update_args)?;
    if update_args.update_self_and_buckets {
        println!("{}", "开始更新hp和buckets".dark_cyan().bold());
        update_hp(&options)?;
        update_buckets().await?;
        return Ok(());
    }
    if update_args.all {
        log::trace!("update all app ");
        update_all_apps(&options)?;
        return Ok(());
    }
    if update_args.app_name.is_none() {
        return Ok(());
    }
    let app_name = update_args.app_name.unwrap();
    log::trace!("update app: {}", app_name);

    update_specific_app(& app_name ,&options )?;
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

    Ok(options)
}

pub(crate) async fn update_buckets() -> Result<(), anyhow::Error> {
    update_scoop_bar().await?; 
    let status = check_bucket_update_status()?; 
     if !status   { return Ok(()); } 
    update_all_buckets_bar()?;
    update_scoop_config_last_update_time();
    Ok(())
}

pub fn update_hp(options: &[UpdateOptions]) -> Result<(), anyhow::Error> {
    update_specific_app("hp"  , options  )?;
    Ok(())
}
