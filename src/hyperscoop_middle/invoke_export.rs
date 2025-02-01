use command_util_lib::export::* ;
use crate::command_args::export::ExportArgs;

pub fn execute_export_command  (file: ExportArgs) -> Result < ()  , anyhow::Error >{
  if let Some(file_name) = file.file_name {
    log::info!("Exporting to {}", file_name);
    if  file_name.contains(  '\\') || file_name.contains('/') { 
      export_config_to_path (file_name.clone()) ;
      if  file.config {
        log::info!("Exporting Scoop config"); 
        export_config_to_path_width_config (file_name) ; 
      }
    } else { 
      export_config_to_current_dir (file_name.clone()) ; 
      if  file.config {
        log::info!("Exporting Scoop config"); 
        export_config_to_current_dir_with_config  (file_name) ;
      }
    }
   
  }
    Ok(())
}
