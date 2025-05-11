use crate::command_args::export::ExportArgs;
use command_util_lib::export::*;

pub fn execute_export_command(file: ExportArgs) -> Result<(), anyhow::Error> {
    if let Some(file_name) = file.file_name {
        log::info!("Exporting to {}", file_name);
        if file_name.contains('\\') || file_name.contains('/') {
            if file.config {
                log::info!("Exporting Scoop config to {}", file_name); 
                export_config_to_path_width_config(file_name.clone())?;
            } else {
                export_config_to_path(file_name.clone())?;
            }
        } else {
            if file.config {
                log::info!("Exporting Scoop config to current directory");
                export_config_to_current_dir_with_config(file_name.clone())?;
            } else {
                export_config_to_current_dir(file_name.clone())?;
            };
        }
    }
    Ok(())
}
