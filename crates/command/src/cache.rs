use crate::i18n::tr;
use crate::init_env::{get_cache_dir_path, get_cache_dir_path_global};
use crate::tr_fmt;
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use std::path::Path;

pub fn display_all_cache_info(is_global: bool) -> anyhow::Result<()> {
    let cache_dir = if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    };
    if !Path::new(&cache_dir).exists() {
        bail!(tr_fmt!(
            "Cache directory does not exist: {dir}",
            "缓存目录不存在: {dir}",
            dir = cache_dir
        ));
    }
    let cache_files =
        std::fs::read_dir(&cache_dir).context("Failed to read cache root directory ")?;
    let mut infos = Vec::new();
    let mut count = 0;
    for file in cache_files {
        let entry = file.context("Failed to read cache file")?;
        let file_path = entry.path();
        let Some(file_name) = file_path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let parts: Vec<&str> = file_name.split('#').collect();
        if parts.len() < 2 {
            continue;
        }
        let app_name = parts[0].to_string();
        let version = parts[1].to_string();
        let zip_size = match std::fs::metadata(&file_path) {
            Ok(m) => m.len() as f64 / 1024.0 / 1024.0,
            Err(_) => continue,
        };
        log::info!("cache file: {}", &app_name);
        log::info!("cache path: {}", file_path.display());
        log::info!("cache size: {} MB", &zip_size);
        infos.push((app_name, version, zip_size));
        count += 1;
    }
    let total_size = infos.iter().fold(0f64, |acc, x| acc + x.2);
    let total_size_parsed = format!("{:.2}", total_size);
    println!(
        "{}",
        tr_fmt!(
            "Total: {count} Files, {size} MB\n",
            "总计: {count} 个文件, {size} MB\n",
            count = count,
            size = total_size_parsed
        )
        .dark_yellow()
        .bold()
    );
    if count == 0 {
        return Ok(());
    }
    println!(
        "{:<30}\t\t{:<30}\t\t{:<30}",
        tr("Name", "名称").green().bold(),
        tr("Version", "版本").green().bold(),
        tr("Size", "大小").green().bold()
    );
    println!(
        "{:<30}\t\t{:<30}\t\t{:<30}",
        "____".green().bold(),
        "_______".green().bold(),
        "____".green().bold()
    );

    println_cache_info(&infos);
    Ok(())
}

fn println_cache_info(app_name: &Vec<(String, String, f64)>) {
    for info in app_name {
        let zip_size_parsed = format!("{:.2}", info.2);
        println!(
            "{:<15} {:<15} {:<15}",
            info.0,
            info.1,
            zip_size_parsed + " MB"
        );
    }
}

pub fn display_specified_cache_info(app_name: &str, is_global: bool) -> anyhow::Result<()> {
    let cache_dir = if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    };
    if !Path::new(&cache_dir).exists() {
        bail!(tr_fmt!(
            "Cache directory does not exist: {dir}",
            "缓存目录不存在: {dir}",
            dir = cache_dir
        ));
    }
    if app_name.is_empty() || app_name.trim() == "*" {
        rm_cache_file(cache_dir)?;
        return Ok(());
    }
    log::info!("display_specified_cache_info : {}", app_name);
    let cache_files =
        std::fs::read_dir(&cache_dir).context("Failed to read cache root directory")?;
    let mut size = 0f64;
    let mut flag = false;
    for file in cache_files {
        let entry = file.context("Failed to read cache file")?;
        let file_path = entry.path();
        let Some(path_name) = file_path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let app = path_name.split('#').next().unwrap_or("").to_string();
        if app.trim().to_lowercase() == app_name {
            size += match std::fs::metadata(&file_path) {
                Ok(m) => m.len() as f64 / 1024.0 / 1024.0,
                Err(_) => 0.0,
            };
            println!(
                "{}",
                tr_fmt!(
                    "Removing cache file: {name}",
                    "正在清理缓存文件: {name}",
                    name = path_name.green().bold()
                )
            );
            std::fs::remove_file(&file_path).context("Failed to remove cache file")?;
            flag = true;
        }
    }
    if !flag {
        bail!(tr_fmt!(
            "Cache for '{app_name}' does not exist",
            "应用 '{app_name}' 的缓存不存在",
            app_name = app_name
        ));
    }
    let size = format!("{:.2}", size);
    println!(
        "{}",
        tr_fmt!(
            "Deleted: 1 File, {size} MB",
            "已删除: 1 个文件, {size} MB",
            size = size
        )
        .dark_yellow()
        .bold()
    );
    Ok(())
}

pub fn rm_all_cache(is_global: bool) -> anyhow::Result<()> {
    let cache_dir = if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    };
    if !Path::new(&cache_dir).exists() {
        bail!(tr_fmt!(
            "Cache directory does not exist: {dir}",
            "缓存目录不存在: {dir}",
            dir = cache_dir
        ));
    }
    rm_cache_file(cache_dir)?;
    Ok(())
}

fn rm_cache_file(cache_dir: String) -> anyhow::Result<()> {
    let mut count = 0;
    let mut size = 0f64;
    for entry in std::fs::read_dir(cache_dir).context("Failed to read cache directory")? {
        let path = entry?.path();
        if path.is_file() {
            size += (std::fs::metadata(&path)
                .context("Failed to read cache file metadata")?
                .len() as f64)
                / 1024f64
                / 1024f64;
            let file_name_display = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            println!(
                "{}",
                tr_fmt!(
                    "Removing cache file: {name}",
                    "正在清理缓存文件: {name}",
                    name = file_name_display.green().bold()
                )
            );
            std::fs::remove_file(&path).context("Failed to remove cache file")?;
            count += 1;
            log::warn!("cache file : {}", path.to_string_lossy().to_string());
        }
    }
    let size = format!("{:.2}", size);
    println!(
        "{}",
        tr_fmt!(
            "Deleted: {count} Files, {size} MB",
            "已删除: {count} 个文件, {size} MB",
            count = count,
            size = size
        )
        .dark_yellow()
        .bold()
    );

    Ok(())
}
