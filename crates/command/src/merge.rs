use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{read_to_string, BufReader};
use std::path::{Path, PathBuf};
use crossterm::style::Stylize;
use crate::buckets::get_buckets_path;
use std::process::exit;
use anyhow::anyhow;
use log::error;
use serde_json;
#[derive(Debug, Eq, PartialEq, Hash)]
struct Merge {
  pub app_name: String,
  pub app_version: String,
}

impl Merge {
  pub fn new(app_name: &String, app_version: &String)
             -> Self { Merge { app_name: app_name.clone(), app_version: app_version.clone() } }
}


// 合并所有冗余的manifest
pub fn merge_all_buckets() -> Result<(), anyhow::Error> {
  //  1. 读取所有bucket的manifest文件
  println!("{ }", "正在合并所有冗余的manifest文件".dark_green().bold());
  let paths = get_buckets_path()?;
  let paths = paths.iter().map(|item|
    item.to_string() + "\\bucket").collect::<Vec<String>>();
  let all_bucket_info: Vec<Merge> = Vec::new();
  // println!("{:?}", paths);
  for path in paths {
    let path_dir = Path::new(&path);
    if path_dir.is_dir() {
      load_bucket_info(path_dir)?;
    }
  }
  println!("{ }", "合并完成".dark_green().bold());

  Ok(())
}

fn load_bucket_info(path_dir: &Path) -> Result<(), anyhow::Error> {
  println!("加载bucket：{}", &path_dir.to_str().expect("Invalid path").to_string().dark_blue().bold());
  for entry in path_dir.read_dir()? {
    let entry = entry?;
    let file_name = entry.file_name();
    // println!("文件名是：{}", file_name.to_str().unwrap().to_string().dark_blue().bold());
    let path = entry.path();
    if path.is_dir() {
      // println!("{ } {} ", "跳过目录".dark_green().bold(),
      //          file_name.to_str().expect("Invalid file name").to_string().dark_blue().bold());
      continue;
    } else if path.is_file() && exclude_not_json_file(file_name.to_string_lossy().to_string()) {
      // println!("{ } {}", "跳过非json文件".dark_green().bold(),
      //          file_name.to_str().unwrap().to_string().dark_blue().bold());
      continue;
    } else if path.is_file() && path.extension().is_some()
      && path.to_string_lossy().to_string().ends_with(".json")
    {
      // 对于 path使用ends_with 只能匹配路径的最后一个元素,不能匹配扩展名
      // println!("{ } {}", "正在读取文件".dark_green().bold(), file_name.to_str().unwrap().to_string().dark_blue().bold());

      let result = extract_info_from_manifest(&path)?;
      execute_merge(&result)?;
      continue;
    } else {
      print!("{}", path.to_str().unwrap().to_string().dark_blue().bold());
      error!("文件类型不支持");
      return Err(anyhow!("文件不存在"));
    }
  }
  Ok(())
}

fn execute_merge(merge: &Merge) -> Result<(), anyhow::Error> {
  // 存入集合
  // println!("    正在合并");
  let mut all_bucket_set: HashMap<String, &Merge> = HashMap::new();

  //如果merge任意字段为空，则跳过
  if merge.app_name.is_empty() || merge.app_version.is_empty()
    || merge.app_name.contains("null")
    || merge.app_version.contains("null") {
    println!("{}  :  {}", merge.app_name.clone().dark_blue().bold(), "无效的manifest".dark_blue().bold());
    return Ok(());
  }

  if !all_bucket_set.contains_key(&merge.app_name) {
    // println!("    正在添加到集合 {}", merge.app_name.clone().dark_green().bold());
    // 返回 Option<&V>  需要ok_or转换为Result
    let result = all_bucket_set.insert(merge.app_name.to_string(), merge);
    //  insert插入的键不存在时，返回None,所有不能进行错误处理
    // .unwrap(); //.ok_or(anyhow!("No app version")).expect("添加到集合失败");
  } else {
    print!("第一个冗余manifest");
    let old_bucket = all_bucket_set.get(&merge.app_name).ok_or(anyhow!("No app version"))
      .expect("不存在这个merge ");
    let old_app_version = old_bucket.app_version.to_string();
    let new_app_versio = merge.app_version.to_string();
    //  insert 会自动覆盖旧值
    println!("{}  :  {}  已经存在, 新版本号：{}  旧版本号：{}"
             , merge.app_name.clone().dark_blue().bold()
             , merge.app_version.clone().dark_blue().bold()
             , new_app_versio.clone().dark_blue().bold()
             , old_app_version.clone().dark_blue().bold());
    if (new_app_versio > old_app_version) {
      let new_bucket = all_bucket_set.insert(merge.app_name.to_string(), merge);
      //.ok_or(anyhow!("No app version")).expect("No app version");
      println!(" 已经更新为最新版本");
    }
    exit(0);
  };

  Ok(())
}


fn extract_info_from_manifest(path: &PathBuf) -> Result<Merge, anyhow::Error> {
  // println!("正在读取文件：{}", path.to_str().unwrap().to_string().dark_blue().bold());

  // 读取文件内容
  let file_buffer = File::open(path).map_err(|e| anyhow::anyhow!("无法打开文件: {}", e))?;
  // 读取文件内容到字符串
  let mainifest_info = read_to_string(file_buffer).map_err(|e| anyhow::anyhow!("无法读取文件: {}", e))?;
  //清理无效的控制字符
  let manifest_json: serde_json::Value = serde_json::from_str(&mainifest_info).map_err(
    |err| println!("文件读取错误 ,{}, 路径是：{}", err, path.to_string_lossy())
  ).unwrap_or_default(); // 空json


  let app_version = manifest_json["version"].to_string();
  // file_stem 去掉文件的扩展名
  let app_name = path.file_stem().expect("Invalid path").to_string_lossy()
    .split("/").last().expect("Invalid path").to_string();
  // println!("{}   :  {}", app_name.clone().dark_blue().bold()
  //          , app_version.clone().dark_blue().bold());
  let merge = Merge::new(&app_name, &app_version);
  Ok(merge)
}

fn exclude_not_json_file(file_name: String) -> bool {
  // 排除非json文件 , 匹配 .开头和_开头的文件
  if file_name.starts_with(".") || file_name.starts_with("_") {
    return true;
  } else if !file_name.ends_with(".json") {
    return true;
  }
  return false;
}
