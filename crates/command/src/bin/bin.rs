#![deny(clippy::shadow)]

use command_util_lib::init_hyperscoop;
use command_util_lib::manifest::search_manifest::SearchManifest;
use command_util_lib::utils::detect_encoding::{
    transform_to_only_version_manifest, transform_to_search_manifest_object,
};
use command_util_lib::utils::get_file_or_dir_metadata::get_dir_updated_time;
use command_util_lib::utils::repair_dirty_json::{fix_dirty_json, DEMO_JSON};
use rayon::iter::ParallelBridge;
use serde_json::{from_str, Value};
use std::cell::RefCell;
use std::fs;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::exit;

fn main() {
    // 开始计时
    let start_time = std::time::Instant::now();
    test_invoke_uninstall_hook_script();
    let end_time = std::time::Instant::now();
    println!("程序运行时间：{:?}", end_time.duration_since(start_time));
}
fn find_manifest_same_version_count_more_than_one() {
    let path = r"C:\Users\superuse\super\version_log";

    let mut count = Vec::new();
    let contents = read_to_string(path).unwrap();
    let arr = contents.split("?").collect::<Vec<&str>>();
    for obj in arr {
        if !obj.trim().is_empty() {
            let obj = obj.trim();
            let obj: Value = from_str(&obj).unwrap();
            for (key, value) in obj.as_object().unwrap() {
                {
                    if value.as_u64().unwrap() > 1 {
                        count.push(obj.to_string());
                    }
                }
            }
            println!("  {:?}", count);
        }
    }
}

fn test_update_time() {
    let dir = "A:/Scoop/buckets/okibcn";
    let file = r"A:\Scoop\apps\scc\3.4.0\LICENSE";
    let time = get_dir_updated_time(PathBuf::from(file).as_ref());
    println!("dir: {}, time: {}", dir, time);
}

fn test_manifest() {
    fn read_manifest_with_fs_read_to_string() {
        //程序运行时间：483.5µs
        let path = PathBuf::from(r"A:\Scoop\buckets\okibcn\bucket\7ztm.json");
        let contents = read_to_string(path).unwrap();
        let trimmed = contents.trim_start_matches('\u{feff}');

        let result: SearchManifest = serde_json::from_str(trimmed).unwrap();
        println!("version: {}", result.get_version().unwrap());

        // let result: SearchManifest = transform_to_only_version_manifest(path.as_ref()).unwrap();
        // let version = result.version.unwrap(); //437µs
        // println!("version: {}", version);
    }
    fn read_mainifest_with_io_read_to_string() {
        //程序运行时间：334.5µs
        let path = PathBuf::from(r"A:\Scoop\buckets\okibcn\bucket\7ztm.json");

        let file = File::open(path).unwrap();
        let mut contents = String::new();
        let mut reader = BufReader::new(&file);
        reader.read_to_string(&mut contents).unwrap();
        let result: SearchManifest = serde_json::from_str(contents.as_str()).unwrap();
        println!("version: {}", result.get_version().unwrap());
    }
    read_mainifest_with_io_read_to_string();
    fn read_manifest_with_io_read_to_end() {
        //程序运行时间：461.1µs
        let path = PathBuf::from(r"A:\Scoop\buckets\okibcn\bucket\qualitymuncher.json");

        let path = PathBuf::from(r"A:\Scoop\buckets\okibcn\bucket\7ztm.json");

        let mut bytes = Vec::new();
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();

        let result: SearchManifest = serde_json::from_slice(&bytes).unwrap();
        println!("version: {}", result.get_version().unwrap());
    }
}

fn compare_version() {
    let version1 = "1.2.3";
    let version2 = "1.2.4";
    let version3 = "1.0.99";
    let version = "140";
    let version4 = "0.30";
    let all_version = [version1, version2, version3];
    let max_version = all_version.iter().max().unwrap();
    println!("max_version: {}", max_version);
}

fn test_encoding_transform() {
    let gbk_file = r"C:\Users\superuse\super\error_log.txt";
    let utf8_bom_file = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/EnableLoopbackUtility.json";

    let utf8 = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/FullscreenPhotoViewer.json";
    let utf = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/fonts-nasu.json";
    let feiqiu = r"A:\Scoop\buckets\apps\bucket\FeiQ.json";
    let hyperscoop = r"A:\Scoop\buckets\Hyperscoop\bucket\Hyperscoop.json";
    let pwsh = r"A:\Scoop\buckets\okibcn_ScoopMaster\bucket\PowerShell-installer.json";
    let clash = r"A:\Scoop\buckets\okibcn_ScoopMaster\bucket\clash_for_windows.json";

    let result = transform_to_search_manifest_object(PathBuf::from(pwsh).as_ref()).unwrap();
    println!("desc: {}", result.version.unwrap());
}

fn test_json_parser() {
    let file = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/EnableLoopbackUtility.json";
    let file1 = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/FullscreenPhotoViewer.json";
    // let test = r"A:\Scoop\buckets\anderlli0053_DEV-tools\bucket\EnableHybernate.json";
    let result = transform_to_search_manifest_object(PathBuf::from(file1).as_ref()).unwrap();
    println!("result: {}", result.version.unwrap());
}

fn test_fix_json() {
    let result = fix_dirty_json(DEMO_JSON).unwrap();
    println!("result: {}", result);
}

fn test_invoke_uninstall_hook_script() {
    use rayon::prelude::*;
    use std::fs;
    let hp = init_hyperscoop().unwrap();
    let mut count = 0;
    let bucket_path = hp.get_bucket_path();
    fs::read_dir(bucket_path)
        .unwrap()
        .par_bridge()
        .for_each(|entry| {
            let entry = entry.unwrap();
            let bucket = entry.path().join("bucket");

            fs::read_dir(bucket)
                .unwrap()
                .par_bridge()
                .for_each(|entry| {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let count = RefCell::new(0);
                    if path.is_file()
                        && path.extension().unwrap_or(std::ffi::OsStr::new("")) == "json"
                    {
                        let contents = fs::read_to_string(&path).unwrap_or_default();
                        let manifest: serde_json::Value =
                            serde_json::from_str(&contents).unwrap_or_default();
                        let post_uninstall = manifest["post_uninstall"].as_array();
                        let pre_uninstall = manifest["pre_uninstall"].as_array();
                        let uninstaller = manifest["uninstaller"].as_str();
                        let installer = manifest["installer"].as_str();
                        let psmodule = manifest["psmodule"].as_object();
                        let env_set = manifest["env_set"].as_object();
                        let env_add_path = manifest["env_add_path"].as_array();
                        if env_set.is_some() && !env_set.unwrap().is_empty() {
                            dbg!(path.display());
                            println!("env {:?}", env_set.unwrap());
                            exit(0);
                        }
                    }
                });
        });
}
