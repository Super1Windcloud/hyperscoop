use crate::command_args::import::ImportArgs;
use anyhow::Context;
use command_util_lib::import::*;
use serde_json::{Map, Value};

pub fn execute_import_command(args: ImportArgs) -> Result<(), anyhow::Error> {
    if let Some(path) = args.path {
        log::info!("{:?}", &path);
        let contents = std::fs::read_to_string(&path).expect("文件编码格式错误或路径错误");
        let config_obj: Value = serde_json::from_str(&contents).expect("配置文件格式错误");
        let default_obj: Map<String, Value> = Map::new();
        let default_arr: Vec<Value> = Vec::new();
        let config = config_obj["config"].as_object().unwrap_or(&default_obj);
        let buckets = config_obj["buckets"].as_array().unwrap_or(&default_arr);
        let apps = config_obj["apps"].as_array().unwrap_or(&default_arr);
        if !config.is_empty() {
            let config_str = serde_json::to_string_pretty(config)
                .context("Failed to convert config object to string")?;
            write_into_scoop_config(config_str);
        }
        if !buckets.is_empty() {
            log::info!("{:?}", buckets.len());
            let mut bucket_info = Vec::new();
            for bucket_obj in buckets {
                let source = bucket_obj["Source"].as_str().unwrap_or_default();
                let name = bucket_obj["Name"].as_str().unwrap_or_default();
                bucket_info.push((name, source));
            }
            add_buckets(bucket_info, path.clone())?;
        }
        if !apps.is_empty() {
            log::info!("{:?}", apps.len());
            let mut app_info = Vec::new();
            for app in apps {
                let app_name = app["Name"].as_str().unwrap_or_default();
                let bucket = app["Source"].as_str().unwrap_or_default();
                let version = app["Version"].as_str().unwrap_or_default();
                app_info.push((app_name, bucket, version));
            }
            install_apps(app_info, path)?;
        }
    }
    Ok(())
}
