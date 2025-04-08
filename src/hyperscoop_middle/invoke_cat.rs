use crate::command_args::cat::CatArgs;
use command_util_lib::buckets::get_buckets_path;
use command_util_lib::cat::catch_manifest;
pub fn execute_cat_command(cat: CatArgs) -> Result<(), anyhow::Error> {
  
    
    if cat.app_name.is_empty() {
        eprintln!("No command provided. Run `hp  --help` to see available commands.");
        return Ok(());
    }
    let app_name = cat.app_name.clone();
    let bucket_paths = get_buckets_path().expect("Failed to get buckets path");
    #[cfg(debug_assertions)]
    dbg!(&app_name);
    log::info!("info : {:?}", &app_name);
    catch_manifest(bucket_paths, app_name)? ; 
  
    Ok(())
}
