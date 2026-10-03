use crate::command_args::desc::DescArgs;
use crate::i18n::{tr, tr_fmt};
use anyhow::bail;
use command_util_lib::init_env::{get_all_buckets_dir_path, get_app_dir_manifest_json};
use crossterm::style::Stylize;
use rayon::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub fn execute_desc_command(args: DescArgs) -> anyhow::Result<()> {
    let bucket_paths = get_all_buckets_dir_path().unwrap_or_default();
    let query_lower = args.query.to_lowercase();

    if !args.search {
        // First check locally installed manifest
        let installed_manifest = get_app_dir_manifest_json(&args.query);
        if Path::new(&installed_manifest).exists() {
            if let Ok(content) = fs::read_to_string(&installed_manifest) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    let desc = json
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("No description available");
                    println!("{}: {}", args.query.bold(), desc);
                    return Ok(());
                }
            }
        }

        // Search bucket manifests
        let mut found_desc = None;
        for bucket_path in &bucket_paths {
            let child = Path::new(bucket_path).join("bucket");
            let dir = if child.is_dir() {
                child
            } else {
                PathBuf::from(bucket_path)
            };
            let manifest_path = dir.join(format!("{}.json", args.query));
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        let desc = json
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("No description available");
                        found_desc = Some(desc.to_string());
                        break;
                    }
                }
            }
        }

        if let Some(desc) = found_desc {
            println!("{}: {}", args.query.bold(), desc);
        } else {
            bail!(
                "{}",
                tr_fmt!(
                    "No manifest found for '{name}'.",
                    "未找到应用 '{name}' 的 manifest 清单。",
                    name = args.query
                )
            );
        }
    } else {
        println!(
            "{}",
            tr_fmt!(
                "Searching descriptions matching '{query}':",
                "正在搜索描述中包含 '{query}' 的应用：",
                query = args.query
            )
            .dark_cyan()
            .bold()
        );

        let matches = Mutex::new(Vec::new());

        bucket_paths.par_iter().for_each(|bucket_path| {
            let child = Path::new(bucket_path).join("bucket");
            let dir = if child.is_dir() {
                child
            } else {
                PathBuf::from(bucket_path)
            };
            if let Ok(entries) = fs::read_dir(dir) {
                entries.par_bridge().for_each(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file()
                            && path.extension().and_then(|s| s.to_str()) == Some("json")
                        {
                            if let Some(app_name) = path.file_stem().and_then(|s| s.to_str()) {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                        if let Some(desc) =
                                            json.get("description").and_then(|d| d.as_str())
                                        {
                                            if desc.to_lowercase().contains(&query_lower) {
                                                if let Ok(mut list) = matches.lock() {
                                                    list.push((
                                                        app_name.to_string(),
                                                        desc.to_string(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });

        let mut results = matches.into_inner().unwrap_or_default();
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results.dedup_by(|a, b| a.0 == b.0);

        if results.is_empty() {
            println!(
                "{}",
                tr(
                    "No matching application descriptions found.",
                    "未找到包含该关键词的应用描述。"
                )
                .dark_grey()
            );
        } else {
            for (app, desc) in results {
                println!("{}: {}", app.bold(), desc);
            }
        }
    }

    Ok(())
}
