mod init_env;
use anyhow;
use command_util_lib::init_hyperscoop;
use init_env::HyperScoop;
use std::env;
use std::path::PathBuf;
mod utils;

use utils::detect_encoding::read_str_from_json;
mod buckets;
fn main() {
  // let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
  // let bucket = buckets::Buckets::new();
  compare_version();
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


fn test_json_parser() {
  let file = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/EnableLoopbackUtility.json";
  let file1 = "A:/Scoop/buckets/anderlli0053_DEV-tools/bucket/FullscreenPhotoViewer.json";
  let result = read_str_from_json(PathBuf::from(file).as_ref()).unwrap();
  let result1 = read_str_from_json(PathBuf::from(file1).as_ref()).unwrap();
  println!("result: {}", result);
  println!("result1: {}", result1);
}
