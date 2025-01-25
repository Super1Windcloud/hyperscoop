use crate::command_args::home::HomeArgs;
use command_util_lib::buckets::get_buckets_path;
use command_util_lib::home::open_home_page;
pub fn execute_home_command(home: HomeArgs) -> Result<(), anyhow::Error> {
    if let Some(name) = home.name {
        log::info!("execute_home_command called with name: {}", name);
        let bucket_path = get_buckets_path().expect("Failed to get buckets path");
        open_home_page(bucket_path, name);
    }
    Ok(())
}
