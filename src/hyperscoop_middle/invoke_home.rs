use crate::command_args::home::HomeArgs;
use command_util_lib::buckets::get_buckets_path;
use command_util_lib::home::open_home_page;
pub fn execute_home_command(home: HomeArgs) -> Result<(), anyhow::Error> {
    let bucket_path = get_buckets_path()?;
    for name in home.names {
        open_home_page(bucket_path.clone(), name)?;
    }
    Ok(())
}
