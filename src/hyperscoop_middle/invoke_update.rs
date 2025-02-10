use crossterm::style::Stylize;
use crate::command_args::update::UpdateArgs;
use command_util_lib::update::*;
pub  async fn execute_update_command(update_args: UpdateArgs) -> Result<(), anyhow::Error> {
    if update_args.update_self {

        println!("{}", "开始更新hp和buckets".dark_cyan().bold());
        update_hp ()?;
        update_buckets().await? ;
        return Ok(());
    }

    if update_args.all {
        log::trace!("update all app ");
        update_all_apps()?;
        return Ok(());
    }
    if update_args.app_name.is_some() {
        let app_name = update_args.app_name.unwrap();
        log::trace!("update app: {}", app_name);
      if update_args.no_cache && update_args.skip_hash_check {
           update_specific_app_without_cache_and_hash_check(app_name.clone())?;
         return Ok(());
      }
      if  update_args.no_cache {
         update_specific_app_without_cache(app_name.clone())?;
        return Ok(());
      }
      if  update_args.skip_hash_check {
        update_specific_app_without_hash_check(app_name.clone() )?;
        return Ok(());
      }
      update_specific_app(app_name.clone())?;
      return Ok(());

    }

    Ok(())
}

async fn update_buckets() -> Result<(), anyhow::Error> {
    log::trace!("Calling update_buckets()"); 
  
  // 更新hp 和buckets
  // 只对官方维护的bucket进行更新
   update_scoop_bar().await ? ; 
    Ok(())
}

fn update_hp() -> Result<(), anyhow::Error> {
    log::trace!("Calling update_scoop()");
  update_specific_app("hp".into())? ;
  Ok(())
}
