use crate::buckets::Buckets;
use crate::list::get_all_installed_apps_name;
use crate::utils::detect_encoding::transform_to_only_version_manifest;
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::fs::DirEntry;
use std::path::{Path, PathBuf}; // 并行处理
pub fn fuzzy_search(query: String) {
    let query = query.trim().to_string().to_lowercase();

    // 存储满足条件的应用名称
    // 使用 ARC和 Mutex 实现多线程安全互斥锁会影响性能，所有这里放弃

    // let mut result = Arc::new(Mutex::new(Vec::new()));
    //
    // buckets_path.par_iter().for_each(|entry| {
    //   let path = Path::new(&entry);
    //   if path.is_dir() {
    //     let bucket_path = path.join("bucket");
    //     if bucket_path.is_dir() {
    //       let temp = get_apps_names(&bucket_path, &query).expect("Failed to get apps info");
    //       if !temp.is_empty() {
    //         let mut result = result.lock().unwrap();
    //         result.extend(temp);
    //       }
    //     }
    //   }
    // });
    // let result = Arc::try_unwrap(result).unwrap().into_inner().unwrap();
    let buckets = Buckets::new();
    let buckets_path = buckets.buckets_path;
    if query.contains("/") {
        let query_args = query
            .clone()
            .split("/")
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        if query_args.len() == 2 {
            search_app_in_specific_bucket(buckets_path, &query_args[0], &query_args[1]);
        }
    } else {
        let all_manifests_path = get_all_manifest_package_name(buckets_path);
        if all_manifests_path.is_err() {
            eprintln!("Error: All manifests path is invalid");
            return;
        }
        let match_result = get_match_name_path(&all_manifests_path.unwrap(), query);
        if match_result.is_err() {
            eprintln!("Error: Failed to get match result");
            return;
        }
        let match_result = match_result.unwrap();
        let result_info = get_result_source_and_version(match_result).unwrap();
        let count = result_info.len();
        println!(
            "\t{} {}",
            count.to_string().dark_green().bold(),
            "Results from local buckets...\n".dark_green().bold()
        );
        sort_result_by_bucket_name(result_info);
    }
}

fn get_match_name_path(
    paths: &Vec<PathBuf>,
    query: String,
) -> Result<Vec<(String, PathBuf)>, anyhow::Error> {
    let result = paths
        .into_par_iter()
        .filter_map(|path| {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            if file_name == query || file_name.contains(&query) {
                Some((file_name.to_string(), path.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(result)
}

pub fn exact_search(query: String) {
    let query = query.trim().to_string().to_lowercase();
    let bucket = Buckets::new();
    let buckets_path = bucket.buckets_path;
    let all_result: Vec<Vec<(String, PathBuf)>> = buckets_path
        .par_iter()
        .filter_map(|entry| {
            let path = Path::new(&entry);
            let bucket_path = path.join("bucket");
            if bucket_path.is_dir() {
                get_exact_search_apps_names(&bucket_path, &query).ok()
            } else {
                None
            }
        })
        .collect(); // 并行处理
    let result = all_result.into_par_iter().flatten().collect();

    let result_info = get_result_source_and_version(result).unwrap();

    let count = result_info.len();
    println!(
        "{} {}",
        count.to_string().dark_green().bold(),
        "Results from local buckets...\n".dark_green().bold()
    );
    sort_result_by_bucket_name(result_info);
}

fn get_result_source_and_version(
    app_names: Vec<(String, PathBuf)>,
) -> Result<Vec<(String, String, String)>, anyhow::Error> {
    let result_info = app_names
        .par_iter()
        .filter_map(|item| {
            let path = &item.1;
            let app_name = item.0.clone();
            let source = path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let version = transform_to_only_version_manifest(path.as_ref());

            if version.is_err() {
                eprintln!(
                    "{}",
                    format!(
                        "{} {}",
                        "Failed to get version of".red(),
                        path.to_string_lossy().to_string().red()
                    )
                    .bold()
                );
                return None;
            }
            let version = version.unwrap();
            let version = version.get_version().unwrap().to_string();
            Some((app_name, version, source))
        })
        .collect();

    Ok(result_info)
}

fn search_app_in_specific_bucket(buckets_path: Vec<String>, bucket: &String, app_name: &String) {
    let path: PathBuf = buckets_path
        .iter()
        .filter_map(|item| {
            if item.contains(bucket) {
                let path = Path::new(item).join("bucket");
                Some(path)
            } else {
                None
            }
        })
        .collect();
    if path.is_dir() {
        let result_name = get_apps_names(&path, app_name).unwrap();
        let result_info = get_result_source_and_version(result_name).unwrap();
        let count = result_info.len();
        println!(
            "{} {}",
            count.to_string().dark_green().bold(),
            "Results from local buckets...\n".dark_green().bold()
        );

        sort_result_by_bucket_name(result_info);
    }
}

fn sort_result_by_bucket_name(mut result: Vec<(String, String, String)>) {
    //  ( name  version source )
    // 将官方维护的bucket靠前
    let official_bucket = ["main", "extras", "versions"];

    for i in 0..result.len() {
        for name in official_bucket {
            if result[i].2 == name {
                result.insert(0, result[i].clone());
                result.remove(i + 1);
                break;
            }
        }
    }

    display_result(&mut result)
}

fn get_apps_names(path: &PathBuf, query: &String) -> Result<Vec<(String, PathBuf)>, anyhow::Error> {
    let query = query.trim().to_string().to_lowercase();
    let app_names = path
        .read_dir()?
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let entry = entry.ok().unwrap();
            let path = entry.path();
            let file_type = entry.file_type().unwrap();
            if file_type.is_file() && path.extension().unwrap_or_default() == "json" {
                let app_name = path.file_stem().unwrap().to_str().unwrap();
                let app_name = app_name.to_string().to_lowercase();
                if app_name == query || app_name.contains(&query) {
                    let app_path = path.clone();
                    Some((app_name.to_string(), app_path))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    Ok(app_names)
}

fn par_read_bucket_dir(path: Vec<String>) -> anyhow::Result<Vec<PathBuf>> {
    let path: Vec<PathBuf> = path
        .into_par_iter()
        .filter_map(|item| {
            let path = Path::new(&item).join("bucket");
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    Ok(path)
}

pub fn get_all_manifest_package_name_slow(
    buckets_path: Vec<String>,
) -> anyhow::Result<Vec<PathBuf>> {
    let buckets_path = par_read_bucket_dir(buckets_path)?;
    let all_manifests_path: Vec<PathBuf> = buckets_path
        .par_iter()
        .filter_map(|item| {
            let dir = std::fs::read_dir(item).ok();
            if dir.is_none() {
                return None;
            }
            let dir = dir.unwrap();
            let paths: Vec<PathBuf> = dir.map(|dir| dir.unwrap().path()).collect();
            Some(paths)
        })
        .collect::<Vec<_>>()
        .iter()
        .flatten()
        .cloned()
        .collect();
    let all_json_manifests: Vec<PathBuf> = all_manifests_path
        .into_par_iter()
        .filter_map(|file_path| {
            // !  千万不要使用 is_file 方法,  path.is_file() 是一个系统调用,系统开销巨大,严重影响性能
            if file_path.is_file() && file_path.extension().unwrap_or_default() == "json" {
                Some(file_path)
            } else {
                None
            }
        })
        .collect();
    // let all_result = all_result.into_par_iter().flatten().collect::<Vec<_>>();
    Ok(all_json_manifests)
}

fn par_read_dir(path: &Path) -> std::io::Result<impl ParallelIterator<Item = DirEntry>> {
    Ok(path.read_dir()?.par_bridge().filter_map(|de| de.ok()))
}

fn is_manifest(dir_entry: &DirEntry) -> bool {
    let filename = dir_entry.file_name();
    let name = filename.to_str().unwrap();
    let is_file = dir_entry.file_type().unwrap().is_file();
    is_file && name.ends_with(".json") && name != "package.json"
}

pub fn get_all_manifest_package_name(buckets_path: Vec<String>) -> anyhow::Result<Vec<PathBuf>> {
    let all_manifests: Vec<PathBuf> = buckets_path
        .into_par_iter()
        .filter_map(|item| {
            let path = Path::new(&item).join("bucket");
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .flat_map(|path| {
            par_read_dir(&path)
                .unwrap()
                .filter(is_manifest)
                .map(|de| de.path())
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(all_manifests)
}

fn get_exact_search_apps_names(
    path: &PathBuf,
    query: &String,
) -> Result<Vec<(String, PathBuf)>, anyhow::Error> {
    let query_lower = query.to_lowercase();
    let app_names = par_read_dir(path)?
        .filter_map(|de| {
            let path = de.path();
            let file_type = de.file_type().ok()?;
            if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json")
            {
                let app_name = path.file_stem().and_then(|stem| stem.to_str())?;
                if app_name.to_lowercase() == query_lower {
                    Some((app_name.to_string(), path))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(app_names)
}

fn display_result(result: &mut Vec<(String, String, String)>) {
     result.sort_by(|a, b|  a.0.cmp(&b.0));
    let name_width = result.iter().map(|(name, _, _)| name.len()).max().unwrap();
    let version_width = result
        .iter()
        .map(|(_, version, _)| version.len())
        .max()
        .unwrap();
    let bucket_width = result
        .iter()
        .map(|(_, _, bucket)| bucket.len())
        .max()
        .unwrap();
  let  total_width = name_width + version_width + bucket_width + 17  ;

  for i in 0..result.len() {
        if i == 0
        {
            println!(" {}", "-".repeat(total_width).dark_magenta().bold()) ;
            println!(
                "{} {:<name_width$ }\t{:<version_width$}\t{:<bucket_width$ } {}",
                 "|".dark_magenta().bold(), 
                "Name",
                "Version",
                "Source","|".dark_magenta().bold(), 
                name_width = name_width + 4,
                version_width = version_width+4  ,
                bucket_width = bucket_width
            );
            println!(
                "{} {:<name_width$ }\t{:<version_width$}\t{:<bucket_width$ } {}",
                 "|".dark_magenta().bold() ,"____",
                "_______",
                "______", "|".dark_magenta().bold(), 
                name_width = name_width + 4,
                version_width = version_width+4,
                bucket_width = bucket_width
            );
        }
        println!(
            "{} {:<name_width$ }\t{:<version_width$}\t{:<bucket_width$ } {}",
             "|".dark_magenta().bold(), result[i].0,
            result[i].1,
            result[i].2,  "|".dark_magenta().bold(), 
            name_width = name_width + 4,
            version_width = version_width+4 ,
            bucket_width = bucket_width
        );

    }
  println!(" {}", "-".repeat(total_width).dark_magenta().bold()) ;
  
}
#[allow(unused)]
fn is_installed(app_name: &str) -> bool {
    let apps_list = get_all_installed_apps_name();
    for item in apps_list {
        if item == app_name {
            return true;
        }
    }
    return false;
}
