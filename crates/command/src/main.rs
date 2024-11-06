#![deny(clippy::shadow)]
mod utils;

use std::path::PathBuf;
use utils::repair_dirty_json::{fix_dirty_json, DEMO_JSON};
use crate::utils::detect_encoding::transform_to_serde_value_object;
use crate::utils::get_file_or_dir_metadata::get_dir_updated_time;

fn main() {

  // 开始计时
  let start_time = std::time::Instant::now();
  let end_time = std::time::Instant::now();
  println!("程序运行时间：{:?}", end_time.duration_since(start_time));
}


fn test_update_time() {
  let dir = "A:/Scoop/buckets/okibcn";
  let file = r"A:\Scoop\apps\scc\3.4.0\LICENSE";
  let time = get_dir_updated_time(PathBuf::from(file).as_ref());
  println!("dir: {}, time: {}", dir, time);
}

fn test_manifest() {
  let path = PathBuf::from("A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/010editor.json");

  // 检查路径是否以 ".json" 结尾
  if path.ends_with("010editor.json") {
    println!("路径以 .json 结尾");
  } else {
    println!("路径不以 .json 结尾");
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

  let result = transform_to_serde_value_object(PathBuf::from(pwsh).as_ref()).unwrap();
  println!("desc: {}", result["version"]);
}


fn test_json_parser() {
  let file = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/EnableLoopbackUtility.json";
  let file1 = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/FullscreenPhotoViewer.json";
  // let test = r"A:\Scoop\buckets\anderlli0053_DEV-tools\bucket\EnableHybernate.json";
  let result = transform_to_serde_value_object(PathBuf::from(file1).as_ref()).unwrap();
  println!("result: {}", result["version"]);
  std::process::exit(0);
  let result1 = transform_to_serde_value_object(PathBuf::from(file).as_ref()).unwrap();
  println!("result1 : {}", result1["version"]);
  std::process::exit(0);


  println!("result: {}", result);
  println!("result1: {}", result1);
}


fn test_fix_json() {
  let result = fix_dirty_json(DEMO_JSON).unwrap();
  println!("result: {}", result);
}
