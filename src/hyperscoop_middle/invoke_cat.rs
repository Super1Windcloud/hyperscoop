use crate::command_args::cat::CatArgs;
use crate::i18n::t;
use command_util_lib::cat::catch_manifest;

pub fn execute_cat_command(cat: CatArgs) -> Result<(), anyhow::Error> {
    if cat.app_names.is_empty() {
        log::debug!("{}", t!("cat.no_app"));
        return Ok(());
    }
    for (idx, app_name) in cat.app_names.iter().enumerate() {
        if idx > 0 {
            println!();
        }
        log::info!("info : {:?}", app_name);
        catch_manifest(cat.global, app_name.clone())?;
    }
    Ok(())
}
