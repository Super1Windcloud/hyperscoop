use crate::init_env::{
    get_app_dir, get_app_dir_global, get_app_dir_manifest_json, get_app_dir_manifest_json_global,
    get_apps_path, get_apps_path_global, get_cache_dir_path, get_cache_dir_path_global,
    get_persist_dir_path, get_persist_dir_path_global,
};
use crate::list::get_install_json_version;
use rayon::prelude::*;
use std::fs::read_dir;
use std::path::Path;

/// Format bytes into human-readable string (B, KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Recursively calculate the total byte size of a directory
pub fn calculate_dir_size<P: AsRef<Path>>(path: P) -> u64 {
    let p = path.as_ref();
    if !p.exists() {
        return 0;
    }
    if p.is_file() {
        return p.metadata().map(|m| m.len()).unwrap_or(0);
    }

    let mut total = 0;
    if let Ok(entries) = read_dir(p) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += calculate_dir_size(&entry_path);
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

/// Calculate cache file size for a given app
pub fn calculate_cache_size(app_name: &str, is_global: bool) -> u64 {
    let cache_dir_str = if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    };

    let cache_path = Path::new(&cache_dir_str);
    if !cache_path.is_dir() {
        return 0;
    }

    let prefix = format!("{}#", app_name.to_lowercase());
    let mut total = 0;

    if let Ok(entries) = read_dir(cache_path) {
        for entry in entries.flatten() {
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string()
                .to_lowercase();
            if name.starts_with(&prefix) || name == app_name.to_lowercase() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
    }

    total
}

/// Disk usage breakdown for an application
#[derive(Debug, Clone)]
pub struct AppDiskUsage {
    pub app_name: String,
    pub version: String,
    pub is_global: bool,
    pub current_dir_size: u64,
    pub total_app_dir_size: u64,
    pub persist_size: u64,
    pub cache_size: u64,
    pub total_size: u64,
}

/// Query disk usage for a single application
pub fn get_app_disk_usage(app_name: &str, is_global: bool) -> anyhow::Result<AppDiskUsage> {
    let app_dir_str = if is_global {
        get_app_dir_global(app_name)
    } else {
        let user = get_app_dir(app_name);
        if Path::new(&user).is_dir() {
            user
        } else {
            get_app_dir_global(app_name)
        }
    };

    let manifest_json = if is_global {
        get_app_dir_manifest_json_global(app_name)
    } else {
        let user_m = get_app_dir_manifest_json(app_name);
        if Path::new(&user_m).is_file() {
            user_m
        } else {
            get_app_dir_manifest_json_global(app_name)
        }
    };

    let version =
        get_install_json_version(&manifest_json).unwrap_or_else(|_| "unknown".to_string());

    let app_dir = Path::new(&app_dir_str);
    let current_dir = app_dir.join("current");

    let total_app_dir_size = calculate_dir_size(app_dir);
    let current_dir_size = calculate_dir_size(&current_dir);

    let persist_root = if is_global {
        get_persist_dir_path_global()
    } else {
        get_persist_dir_path()
    };
    let persist_path = Path::new(&persist_root).join(app_name);
    let persist_size = calculate_dir_size(&persist_path);

    let cache_size = calculate_cache_size(app_name, is_global);
    let total_size = total_app_dir_size + persist_size + cache_size;

    Ok(AppDiskUsage {
        app_name: app_name.to_string(),
        version,
        is_global,
        current_dir_size,
        total_app_dir_size,
        persist_size,
        cache_size,
        total_size,
    })
}

/// Query disk usage across all installed applications in parallel
pub fn get_all_apps_disk_usage(is_global: bool) -> anyhow::Result<Vec<AppDiskUsage>> {
    let scan_dirs = if is_global {
        vec![(get_apps_path_global(), true)]
    } else {
        vec![(get_apps_path(), false), (get_apps_path_global(), true)]
    };

    let mut all_apps_to_scan = Vec::new();
    for (root_dir, is_g) in scan_dirs {
        let p = Path::new(&root_dir);
        if !p.is_dir() {
            continue;
        }
        if let Ok(entries) = read_dir(p) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.eq_ignore_ascii_case("scoop") {
                    continue;
                }
                all_apps_to_scan.push((name, is_g));
            }
        }
    }

    let mut usages: Vec<AppDiskUsage> = all_apps_to_scan
        .par_iter()
        .filter_map(|(name, is_g)| get_app_disk_usage(name, *is_g).ok())
        .collect();

    usages.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    Ok(usages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GB");
    }
}
