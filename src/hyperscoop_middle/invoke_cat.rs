use crate::command_args::cat::CatArgs;
use crate::i18n::tr;
use command_util_lib::cat::catch_manifest;

pub fn execute_cat_command(cat: CatArgs) -> Result<(), anyhow::Error> {
    if cat.app_name.is_empty() {
        eprintln!(
            "{}",
            tr(
                "No app name provided. Run `hp cat --help` to see usage.",
                "未提供应用名称。运行 `hp cat --help` 查看用法。"
            )
        );
        return Ok(());
    }
    let app_name = cat.app_name.clone();

    log::info!("info : {:?}", &app_name);
    catch_manifest(cat.global, app_name)?;
    Ok(())
}
