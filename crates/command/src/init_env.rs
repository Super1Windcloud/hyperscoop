use crate::install::InstallOptions;
use rayon::prelude::*;
use std::env;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
pub fn init_user_scoop() -> String {
    let mut path = env::var("SCOOP").unwrap_or(String::new());
    if path.is_empty() {
        path = env::var("USERPROFILE").unwrap() + "\\scoop"; // 可以使用or_else 替代
    }
    if !Path::new(&path).exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn init_scoop_global() -> String {
    let path = env::var("SCOOP_GLOBAL").or(env::var("ProgramData"));
    if path.is_err() {
        panic!("No SCOOP_GLOBAL environment variable provided.");
    }
    let path = path.unwrap();
    if !Path::new(&path).exists() {
        std::fs::create_dir_all(&path).unwrap()
    }

    path + "\\scoop"
}

pub fn get_app_current_dir(app_name: &str) -> String {
    let scoop_home = init_user_scoop();
    format!("{}\\apps\\{}\\current", scoop_home, app_name)
}

pub fn get_app_dir(app_name: &str) -> String {
    let scoop_home = init_user_scoop();
    format!("{}\\apps\\{}", scoop_home, app_name)
}
pub fn get_app_version_dir(app_name: &str, version: &str) -> String {
    let scoop_home = init_user_scoop();
    format!("{}\\apps\\{}\\{}", scoop_home, app_name, version)
}
pub fn get_app_dir_install_json(app_name: &str) -> String {
    let scoop_home = init_user_scoop();

    format!("{}\\apps\\{}\\current\\install.json", scoop_home, app_name)
}
pub fn get_app_dir_manifest_json(app_name: &str) -> String {
    let scoop_home = init_user_scoop();
    format!("{}\\apps\\{}\\current\\manifest.json", scoop_home, app_name)
}

pub fn get_app_current_bin_path(app_name: String, bin_name: &String) -> String {
    let scoop_home = init_user_scoop();
    format!("{}\\apps\\{}\\current\\{}", scoop_home, app_name, bin_name)
}

pub fn get_old_scoop_dir() -> String {
    let path = env::var("LocalAppData").unwrap_or(String::new());
    path + "\\scoop"
}

pub fn get_scoop_cfg_path() -> String {
    let path = env::var("USERPROFILE").unwrap();
    path + "\\.config\\scoop\\config.json"
}

#[derive(Debug)]
pub struct HyperScoop {
    pub scoop_path: String,
    pub bucket_path: String,
    pub cache_path: String,
    pub shims_path: String,
    pub persist_path: String,
    pub apps_path: String,
}
#[derive(Debug)]
pub struct HyperScoopGlobal {
    pub scoop_path: String,
    pub bucket_path: String,
    pub cache_path: String,
    pub shims_path: String,
    pub persist_path: String,
    pub apps_path: String,
}

impl HyperScoop {
    pub fn new() -> Self {
        Self {
            scoop_path: init_user_scoop(),
            bucket_path: format!("{}\\buckets", init_user_scoop()),
            cache_path: format!("{}\\cache", init_user_scoop()),
            shims_path: format!("{}\\shims", init_user_scoop()),
            persist_path: format!("{}\\persist", init_user_scoop()),
            apps_path: format!("{}\\apps", init_user_scoop()),
        }
    }
    pub fn get_apps_path(&self) -> String {
        let apps_path = self.apps_path.clone();
        if !Path::new(&apps_path).exists() {
            std::fs::create_dir_all(&apps_path).unwrap();
        }
        apps_path
    }
    pub fn get_psmodule_path(&self) -> String {
        let psmodule = format!("{}\\modules", self.scoop_path);
        if !Path::new(&psmodule).exists() {
            std::fs::create_dir_all(&psmodule).unwrap();
        }
        psmodule
    }
    pub fn get_persist_path(&self) -> String {
        let persist = self.persist_path.clone();
        if !Path::new(&persist).exists() {
            std::fs::create_dir_all(&persist).unwrap();
        }
        persist
    }

    pub fn get_bucket_path(&self) -> String {
        let bucket_path = &self.bucket_path;
        if !Path::new(&bucket_path).exists() {
            std::fs::create_dir_all(&bucket_path).unwrap();
        }
        bucket_path.into()
    }
    pub fn get_cache_path(&self) -> String {
        let cache_path = self.cache_path.clone();
        if !Path::new(&cache_path).exists() {
            std::fs::create_dir_all(&cache_path).unwrap();
        }
        cache_path
    }
    pub fn get_shims_path(&self) -> String {
        let shim_path = self.shims_path.clone();
        if !Path::new(&shim_path).exists() {
            std::fs::create_dir_all(&shim_path).unwrap();
        }
        shim_path
    }
    pub fn get_scoop_path(&self) -> String {
        self.scoop_path.clone()
    }
    pub fn print_all_paths(&self) {
        println!("Scoop Path: {}", self.scoop_path);
        println!("Buckets: {}", self.bucket_path);
        println!("Cache: {}", self.cache_path);
        println!("Shims: {}", self.shims_path);
        println!("Persist: {}", self.persist_path);
        println!("Apps: {}", self.apps_path);
    }
}

impl HyperScoopGlobal {
    pub fn new() -> Self {
        Self {
            scoop_path: init_scoop_global(),
            bucket_path: format!("{}\\buckets", init_scoop_global()),
            cache_path: format!("{}\\cache", init_scoop_global()),
            shims_path: format!("{}\\shims", init_scoop_global()),
            persist_path: format!("{}\\persist", init_scoop_global()),
            apps_path: format!("{}\\apps", init_scoop_global()),
        }
    }
    pub fn get_apps_path(&self) -> String {
        let apps_path = self.apps_path.clone();
        if !Path::new(&apps_path).exists() {
            std::fs::create_dir_all(&apps_path).unwrap();
        }
        apps_path
    }
    pub fn get_psmodule_path(&self) -> String {
        let psmodule = format!("{}\\modules", self.scoop_path);
        if !Path::new(&psmodule).exists() {
            std::fs::create_dir_all(&psmodule).unwrap();
        }
        psmodule
    }
    pub fn get_persist_path(&self) -> String {
        let persist = self.persist_path.clone();
        if !Path::new(&persist).exists() {
            std::fs::create_dir_all(&persist).unwrap();
        }
        persist
    }

    pub fn get_bucket_path(&self) -> String {
        let bucket_path = self.bucket_path.clone();
        if !Path::new(&bucket_path).exists() {
            std::fs::create_dir_all(&bucket_path).unwrap();
        }
        bucket_path
    }
    pub fn get_cache_path(&self) -> String {
        let cache_path = self.cache_path.clone();
        if !Path::new(&cache_path).exists() {
            std::fs::create_dir_all(&cache_path).unwrap();
        }
        cache_path
    }
    pub fn get_shims_path(&self) -> String {
        let shim_path = self.shims_path.clone();
        if !Path::new(&shim_path).exists() {
            std::fs::create_dir_all(&shim_path).unwrap();
        }
        shim_path
    }
    pub fn get_scoop_path(&self) -> String {
        self.scoop_path.clone()
    }
    pub fn print_all_paths(&self) {
        println!("Scoop Path: {}", self.scoop_path);
        println!("Buckets: {}", self.bucket_path);
        println!("Cache: {}", self.cache_path);
        println!("Shims: {}", self.shims_path);
        println!("Persist: {}", self.persist_path);
        println!("Apps: {}", self.apps_path);
    }
}

pub fn get_bucket_dir_path() -> String {
    let hyper_scoop = HyperScoop::new();
    hyper_scoop.get_bucket_path()
}
pub fn get_persist_dir_path() -> String {
    let hyper_scoop = HyperScoop::new();
    hyper_scoop.get_persist_path()
}
pub fn get_cache_dir_path() -> String {
    let hyper_scoop = HyperScoop::new();
    hyper_scoop.get_cache_path()
}

pub fn get_buckets_root_dir_path() -> String {
    let hyper_scoop = HyperScoop::new();
    hyper_scoop.get_bucket_path()
}
pub fn get_shims_path() -> String {
    let hyper_scoop = HyperScoop::new();
    hyper_scoop.get_shims_path()
}

pub fn get_apps_path() -> String {
    let hyper_scoop = HyperScoop::new();
    hyper_scoop.get_apps_path()
}

pub fn get_persist_app_data_dir(app_name: &str) -> String {
    let scoop_user_home = init_user_scoop();
    format!("{}\\persist\\{}", scoop_user_home, app_name)
}

// 全局版本的 get_app_current_dir
pub fn get_app_current_dir_global(app_name: &str) -> String {
    let scoop_home = init_scoop_global();
    format!("{}\\apps\\{}\\current", scoop_home, app_name)
}

// 全局版本的 get_app_dir
pub fn get_app_dir_global(app_name: &str) -> String {
    let scoop_home = init_scoop_global();
    format!("{}\\apps\\{}", scoop_home, app_name)
}

// 全局版本的 get_app_version_dir
pub fn get_app_version_dir_global(app_name: &str, version: &str) -> String {
    let scoop_home = init_scoop_global();
    format!("{}\\apps\\{}\\{}", scoop_home, app_name, version)
}

// 全局版本的 get_app_dir_install_json
pub fn get_app_dir_install_json_global(app_name: &str) -> String {
    let scoop_home = init_scoop_global();
    format!("{}\\apps\\{}\\current\\install.json", scoop_home, app_name)
}

// 全局版本的 get_app_dir_manifest_json
pub fn get_app_dir_manifest_json_global(app_name: &str) -> String {
    let scoop_home = init_scoop_global();
    format!("{}\\apps\\{}\\current\\manifest.json", scoop_home, app_name)
}

// 全局版本的 get_app_current_bin_path
pub fn get_app_current_bin_path_global(app_name: String, bin_name: &String) -> String {
    let scoop_home = init_scoop_global();
    format!("{}\\apps\\{}\\current\\{}", scoop_home, app_name, bin_name)
}

// 全局版本的 get_old_scoop_dir
pub fn get_old_scoop_dir_global() -> String {
    let path = env::var("ProgramData").unwrap_or(String::new());
    path + "\\scoop"
}


// 全局版本的 get_scoop_cfg_path
pub fn get_scoop_cfg_path_global() -> String {
    let path = env::var("ProgramData").unwrap();
    path + "\\scoop\\.config\\config.json"
}

// 全局版本的 get_persist_dir_path
pub fn get_persist_dir_path_global() -> String {
    let hyper_scoop = HyperScoopGlobal::new();
    hyper_scoop.get_persist_path()
}

pub fn get_psmodules_root_global_dir() -> String {
 let  hp = HyperScoopGlobal::new();
   hp.get_psmodule_path()
}

pub fn  get_psmodules_root_dir()->String {
   let  hp= HyperScoop::new(); 
   hp. get_psmodule_path()
}


pub fn get_persist_app_data_dir_global(app_name: &str) -> String {
    let scoop_global_home = init_scoop_global();
    format!("{}\\persist\\{}", scoop_global_home, app_name)
}
// 全局版本的 get_cache_dir_path
pub fn get_cache_dir_path_global() -> String {
    let hyper_scoop = HyperScoopGlobal::new();
    hyper_scoop.get_cache_path()
}

// 全局版本的 get_buckets_root_dir_path
pub fn get_buckets_root_dir_path_global() -> String {
    let hyper_scoop = HyperScoopGlobal::new();
    hyper_scoop.get_bucket_path()
}

// 全局版本的 get_shims_path
pub fn get_shims_path_global() -> String {
    let hyper_scoop = HyperScoopGlobal::new();
    hyper_scoop.get_shims_path()
}

// 全局版本的 get_apps_path
pub fn get_apps_path_global() -> String {
    let hyper_scoop = HyperScoopGlobal::new();
    hyper_scoop.get_apps_path()
}

pub fn get_all_buckets_dir_path() -> anyhow::Result<Vec<String>> {
    let bucket_path = get_bucket_dir_path();
    // 遍历 bucket_path 下的所有文件夹，并将文件夹名加入 buckets_path
    let buckets_path: Vec<String> = read_dir(&bucket_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().to_str().unwrap().to_string())
        .collect();

    Ok(buckets_path)
}
pub fn get_all_buckets_dir_child_bucket_path() -> anyhow::Result<Vec<String>> {
    let bucket_path = get_bucket_dir_path();
    // 遍历 bucket_path 下的所有文件夹，并将文件夹名加入 buckets_path
    let buckets_path: Vec<String> = read_dir(&bucket_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().join("bucket").to_str().unwrap().to_string())
        .collect();

    Ok(buckets_path)
}

pub fn get_all_global_buckets_dir_path() -> anyhow::Result<Vec<String>> {
    let bucket_path = get_buckets_root_dir_path_global();
    let buckets_path: Vec<String> = read_dir(&bucket_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().to_str().unwrap().to_string())
        .collect();
    Ok(buckets_path)
}
pub fn get_all_global_buckets_dir_child_bucket_path() -> anyhow::Result<Vec<String>> {
    let bucket_path = get_buckets_root_dir_path_global();
    let buckets_path: Vec<String> = read_dir(&bucket_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().join("bucket").to_str().unwrap().to_string())
        .collect();
    Ok(buckets_path)
}

pub fn get_scoop_config_path() -> anyhow::Result<String> {
    let home_dir = env::var("USERPROFILE")?;
    let config_dir = home_dir + "\\.config\\scoop";
    if !Path::new(&config_dir).exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    let config_file = format!("{}\\config.json", config_dir);
    if !Path::new(&config_file).exists() {
        std::fs::File::create(&config_file)?;
    }
    Ok(config_file)
}

pub fn get_special_bucket_path(bucket_name: &str) -> String {
    let bucket_root_dir = get_buckets_root_dir_path();
    format!("{}\\{}", bucket_root_dir, bucket_name)
}
pub fn get_special_bucket_child_path(bucket_name: &str) -> String {
    let bucket_root_dir = get_buckets_root_dir_path();
    format!("{}\\{}\\bucket", bucket_root_dir, bucket_name)
}

pub fn get_special_bucket_child_path_global(bucket_name: &str) -> String {
    let bucket_root_dir = get_buckets_root_dir_path_global();
    format!("{}\\{}\\bucket", bucket_root_dir, bucket_name)
}

pub fn get_special_bucket_path_global(bucket_name: &str) -> String {
    let bucket_root_dir = get_buckets_root_dir_path_global();
    format!("{}\\{}", bucket_root_dir, bucket_name)
}

pub fn get_special_bucket_all_manifest_path(bucket_name: &str) -> anyhow::Result<Vec<PathBuf>> {
    let bucket_path = get_special_bucket_child_path(bucket_name);
    let entries =
        read_dir(&bucket_path).expect(format!("Failed to read dir {:?}", bucket_path).as_str());
    let buckets_path = entries
        .par_bridge()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();
    Ok(buckets_path)
}

pub fn get_special_bucket_all_manifest_path_global(
    bucket_name: &str,
) -> anyhow::Result<Vec<PathBuf>> {
    let bucket_path = get_special_bucket_child_path_global(bucket_name);
    let entries =
        read_dir(&bucket_path).expect(format!("Failed to read dir {:?}", bucket_path).as_str());

    let buckets_path = entries
        .par_bridge()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();

    Ok(buckets_path)
}

pub fn check_bucket_whether_exists(
    bucket_name: &str,
    options: &[InstallOptions],
) -> anyhow::Result<bool> {
    let special_bucket_path = if options.contains(&InstallOptions::Global) {
        get_special_bucket_path_global(bucket_name)
    } else {
        get_special_bucket_path(bucket_name)
    };
    if !Path::new(&special_bucket_path).exists() {
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn get_special_version_all_manifest_path_global(
) -> anyhow::Result<Vec<PathBuf>> {
    let all_buckets_dir = get_all_global_buckets_dir_child_bucket_path()?;
    let entries = all_buckets_dir
        .iter()
        .map(|path| read_dir(path).unwrap())
        .collect::<Vec<_>>();

    let buckets_path = entries
        .into_par_iter()
        .map(|dir| dir.par_bridge().filter_map(|e| e.ok()).map(|e| e.path()))
        .flatten()
        .collect::<Vec<_>>();
  
    Ok(buckets_path)
}

pub fn get_special_version_all_manifest_path() -> anyhow::Result<Vec<PathBuf>> {
    let all_buckets_dir = get_all_buckets_dir_child_bucket_path()?;
    let entries = all_buckets_dir
        .iter()
        .map(|path| read_dir(path).unwrap())
        .collect::<Vec<_>>();

    let buckets_path = entries
        .into_par_iter()
        .map(|dir| dir.par_bridge().filter_map(|e| e.ok()).map(|e| e.path()))
        .flatten()
        .collect::<Vec<_>>();
    Ok(buckets_path)
}
mod test_path {
    #[allow(unused)]
    use super::*;
    #[test]
    fn get_current_bin_path() {
        let app_name = "zigmod";
        let exe_name = "zig/zig.exe";
        let path = get_app_current_bin_path(app_name.to_string(), &exe_name.to_string());
        if Path::new(&path).exists() {
            println!("{}", path);
        }
    }

    #[test]
    #[ignore]
    fn test_get_suffix() {
        let _exe_name = "zig/zig.cmd";
        let _exe_name = "zig//zig.cmd";
        let _exe_name = "zig.cmd";
        let _exe_name = r"bin\zig.cmd";
        let exe_name = r"bin\\zig.cmd";
        let suffix = exe_name.split(".").last().unwrap_or("");
        let prefix = exe_name.split(".").next().unwrap_or("");
        println!("{suffix}  {prefix}");
    }

    #[test]
    fn test_global() {
        let path = env::var("ProgramData").unwrap() + "\\scoop";
        println!("{}", path);
    }
    #[test]
    fn test_simple_output() {
        println!("{}", get_scoop_config_path().unwrap());
        println!("{}", get_special_bucket_path("main"));
        println!("{}", get_app_dir("git"));
    }

    #[test]
    fn test_all_manifests() {
        get_special_bucket_all_manifest_path("main").unwrap();
    }
}
