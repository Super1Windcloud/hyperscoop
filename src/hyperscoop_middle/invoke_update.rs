use crate::command_args::update::UpdateArgs;
use log::{info, warn};

pub fn execute_update_command(update_args: UpdateArgs) -> Result<(), anyhow::Error> {
    if update_args.name.is_some() {
    } else {
        // 更新scoop和buckets
        // 只对官方维护的bucket进行更新

        update_scoop();
        update_buckets();
    }

    Ok(())
}

fn update_buckets() {
    warn!("Calling update_buckets()");
}

fn update_scoop() {
    info!("Calling update_scoop()");
}
