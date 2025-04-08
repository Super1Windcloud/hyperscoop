use crate::init_env::{get_cache_dir_path, get_cache_dir_path_global};
use crate::HyperScoop;
use anyhow::bail;
use crossterm::style::Stylize;
use std::path::Path;

pub fn display_all_cache_info(is_global: bool) -> anyhow::Result<()> {
    let cache_dir = if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    };
    if !Path::new(&cache_dir).exists() {
        bail!("cache dir does not exist: {:?}", &cache_dir);
    }
    let cache_files = std::fs::read_dir(cache_dir)?;
    let mut infos = Vec::new();
    let mut count = 0;
    for file in cache_files {
        let path = file?;
        let path1 = path
            .path()
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let path2 = path.path().clone().to_string_lossy().to_string();
        let app_name = path1.split("#").collect::<Vec<&str>>()[0].to_string();
        let zip_size = (std::fs::metadata(&path2)?.len() as f64) / 1024f64 / 1024f64;
        log::info!("cache file : {}", &app_name);
        log::info!("cache file : {}", &path2);
        log::info!("cache size : {} MB", &zip_size);
        let version = path1.split("#").collect::<Vec<&str>>()[1].to_string();
        log::info!("cache version : {}", &version);
        infos.push((app_name, version, zip_size));
        count += 1;
    }
    let total_size = infos.iter().fold(0f64, |acc, x| acc + x.2);
    let total_size_parsed = format!("{:.2}", total_size);
    println!(
        "{} {} {} {} {}\n",
        "Total : ".to_string().yellow().bold(),
        count.to_string().dark_yellow().bold(),
        "Files, ".to_string().yellow().bold(),
        total_size_parsed.to_string().yellow().bold(),
        "MB".to_string().dark_yellow().bold()
    );
    if count == 0 {
        return Ok(());
    }
    println!(
        "{:<30}\t\t{:<30}\t\t{:<30}",
        "Name".green().bold(),
        "Version".green().bold(),
        "Size".green().bold()
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

pub fn display_specified_cache_info(app_name: String, is_global: bool) -> anyhow::Result<()> {
    let cache_dir = if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    };
    if !Path::new(&cache_dir).exists() {
        bail!("cache dir does not exist: {:?}", &cache_dir);
    }
    if app_name.is_empty() || app_name.trim() == "*" {
        rm_cache_file(cache_dir)?;
        return Ok(());
    }
    log::info!("display_specified_cache_info : {}", app_name);
    let cache_files = std::fs::read_dir(cache_dir)?;
    let mut size = 0f64; 
   let mut  flag = false; 
    for file in cache_files {
        let path = file?;
        let t = path.path().clone().to_string_lossy().to_string();
        let path_name = path
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let app = path_name.split("#").collect::<Vec<&str>>()[0].to_string();
        if app == app_name {
            size =
                size + (std::fs::metadata(path.path().clone())?.len() as f64) / 1024f64 / 1024f64;
            println!("Removing cache file : {}", path_name.green().bold());
            std::fs::remove_file(t)?; 
             flag = true; 
        }
    }
  if  !flag {
        bail!("{} cache is not exist ", &app_name);
  }
    let size = format!("{:.2}", size);
    println!(
        "{} {} {}",
        "Deleted  : 1 File,".to_string().yellow().bold(),
        size.to_string().yellow().bold(),
        "MB".yellow().bold()
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
        bail!("cache dir does not exist: {:?}", &cache_dir);
    }
    rm_cache_file(cache_dir)?;
    Ok(())
}

fn rm_cache_file(cache_dir: String) -> anyhow::Result<()> {
    let mut count = 0;
    let mut size = 0f64;
    for entry in std::fs::read_dir(cache_dir)? {
        let path = entry?.path();
        if path.is_file() {
            size += (std::fs::metadata(&path)?.len() as f64) / 1024f64 / 1024f64;
            println!(
                "Removing cache file : {}",
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .green()
                    .bold()
            );
            std::fs::remove_file(&path).expect("Failed to remove file");
            count += 1;
            log::warn!("cache file : {}", path.to_string_lossy().to_string());
        }
    }
    let size = format!("{:.2}", size);
    println!(
        "{} {} {} {} {}",
        "Deleted  : ".to_string().yellow().bold(),
        count.to_string().yellow().bold(),
        "Files, ".to_string().yellow().bold(),
        size.to_string().yellow().bold(),
        "MB".to_string().yellow().bold()
    );

    Ok(())
}
