use crate::init_env::{
    get_all_buckets_dir_child_bucket_path, get_all_global_buckets_dir_child_bucket_path,
};
use anyhow::Context;
use bat::PrettyPrinter;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
fn get_all_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_type = entry.file_type().unwrap();
                if file_type.is_file() {
                    files.push(path);
                }
            }
        }
    }
    files
}

pub fn catch_manifest(global: bool, app_name: String) -> anyhow::Result<()> {
    let bucket_paths = if global {
        get_all_global_buckets_dir_child_bucket_path()?
    } else {
        get_all_buckets_dir_child_bucket_path()?
    };

    let manifest_path = bucket_paths
        .par_iter()
        .flat_map(|bucket_path| {
            let path = Path::new(bucket_path);
            get_all_files(path).into_par_iter()
        })
        .collect::<Vec<PathBuf>>();


    for bucket_path in &manifest_path {
        if Path::new(bucket_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default()
            != app_name.to_lowercase()
        {
            continue;
        }

        let content = fs::read_to_string(bucket_path)
            .context(format!("Failed to read file {} as line 15", bucket_path.display()))?;
        let buffer = content.as_bytes();

        PrettyPrinter::new()
            .input_from_bytes(buffer)
            .language("json")
            .print()?;
        std::process::exit(0);
    }

    Ok(())
}
