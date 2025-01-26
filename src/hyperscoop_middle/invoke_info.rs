use command_util_lib::buckets::get_buckets_path;
use command_util_lib::info::display_app_info;

pub fn execute_info_command(
    info: crate::command_args::info::InfoArgs,
) -> Result<(), anyhow::Error> {
    if info.name.is_some() {
        let app_name = info.name.as_ref().unwrap();
        let bucket_paths = get_buckets_path().expect("Failed to get buckets path");
        display_app_info(app_name.clone(), bucket_paths);
        return Ok(());
    }
    Ok(())
}
