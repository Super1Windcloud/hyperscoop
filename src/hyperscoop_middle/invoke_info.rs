use command_util_lib::buckets::{get_global_all_buckets_dir};
use command_util_lib::info::display_app_info;
use command_util_lib::init_env::get_all_buckets_dir_path;

pub fn execute_info_command(
    info: crate::command_args::info::InfoArgs,
) -> Result<(), anyhow::Error> {
    if info.name.is_some() {
        let app_name = info.name.as_ref().unwrap();
        let bucket_paths = if info.global {
            get_global_all_buckets_dir()?
        } else {
            get_all_buckets_dir_path()?
        };
        display_app_info(app_name.clone(), bucket_paths);
        return Ok(());
    }
    Ok(())
}
