use std::cmp::Ordering;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::merge::Merge;
use crate::utils::system::get_system_current_time;

pub  fn compare_versions(ver1: String, ver2: String) -> Ordering {
  // 分割版本号并转换为数字数组
  let v1: Vec<i32> = ver1.split('.').flat_map(|s| s.parse()).collect();
  let v2: Vec<i32> = ver2.split('.').flat_map(|s| s.parse()).collect();

  // 动态比较每一级（自动补零）
  let max_len = v1.len().max(v2.len());
  (0..max_len)
    .map(|i| (v1.get(i).copied().unwrap_or(0), v2.get(i).copied().unwrap_or(0)))
    .find_map(|(a, b)| match a.cmp(&b) {
      Ordering::Equal => None,
      diff => Some(diff),
    })
    .unwrap_or(Ordering::Equal)
}


pub fn update_scoop_config_last_update_time() {
  let current = get_system_current_time().unwrap();
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  });
  let config_path = Path::new(&config_path);
  if config_path.exists() {
    let config_file = std::fs::File::open(config_path).unwrap();
    let mut config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    if let Some(obj) = config_json.as_object_mut() {
      obj.insert("last_update".into() ,  current.into());
    }
    let file = std::fs::File::create(config_path).unwrap();
    serde_json::to_writer_pretty(file, &config_json).unwrap();
  }
}


pub  fn  get_official_buckets_name()  -> Vec<String> {
  let exclude_dirs = [
    "main",
    "extras",
    "versions",
    "nirsoft",
    "sysinternals",
    "php",
    "nerd-fonts",
    "nonportable",
    "java",
    "games",
  ];
     exclude_dirs.iter().map(|s| s.to_string()).collect()
}

pub fn get_official_bucket_path( bucket_name : String  ) ->String  {
  format!("{}\\buckets\\{}", std::env::var("SCOOP").unwrap_or("USERPROFILE/scoop".into()), bucket_name)
}


pub fn write_into_log_file (path : &PathBuf) {
  let log_file_path = r"A:\Rust_Project\hyperscoop\redundant_log.txt";
  let file = std::fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open(log_file_path)
    .unwrap();
  let mut writer = std::io::BufWriter::new(file);
  writer.write_all((path.to_str().unwrap().to_string()+"\n").as_bytes()).unwrap();
}


pub fn  write_into_log_one_time(msg : &Vec<Merge>) {
   let  log_file = r"A:\Rust_Project\hyperscoop\manifests.txt";
  let  file = fs::File::create(log_file).unwrap();
  let mut writer = std::io::BufWriter::new(file);
  let    mut   str = String::new();
  for  merge  in  msg.iter() {
    str.push_str(&format!("Name :{} Version :{} \n", merge.app_name, merge.app_version));
  }
  writer.write_all(str .as_bytes()).unwrap();
}

pub const LARGE_COMMUNITY_BUCKET: [&str; 8] = [
"https://github.com/anderlli0053/DEV-tools",
"https://github.com/cmontage/scoopbucket",
"https://github.com/duzyn/scoop-cn",
"https://github.com/lzwme/scoop-proxy-cn",
"https://github.com/kkzzhizhou/scoop-apps",
"https://github.com/cmontage/scoopbucket-third",
"https://github.com/okibcn/ScoopMaster",
"http://github.com/okibcn/ScoopMaster",
];
pub fn remove_bom_and_control_chars_from_utf8_file<P: AsRef<Path>>(path: P) -> anyhow::Result<String > {
  // 读取文件内容到字节数组
  let data = fs::read(&path)?;

  // 检查是否存在 BOM（0xEF 0xBB 0xBF）
  let data = if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
    // 截取 BOM 之后的内容
    &data[3..]
  } else {
    &data
  };


  let mut filtered_data = Vec::new();
  let mut idx = 0;

  while idx < data.len() {
    // 跳过控制字符（0x00 到 0x1F 和 0x7F）
    if data[idx] <= 0x1F || data[idx] == 0x7F && !matches!(data[idx], b'\n' | b'\r' | b' ')  {
      idx += 1;
      continue; // 跳过控制字符
    }

    // 尝试解析 UTF-8 字符
    match std::str::from_utf8(&data[idx..]) {
      Ok(s) => {
        if let Some(c) = s.chars().next() {
          // 保留空格、制表符、换行、回车等
          if !c.is_control() || c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            filtered_data.extend_from_slice(&data[idx..idx + c.len_utf8()]);
          }
          idx += c.len_utf8(); // 移动到下一个字符
        } else {
          // 空字符串，直接跳过
          break;
        }
      }
      Err(_) => {
        // 如果解析失败，跳过当前字节
        idx += 1;
      }
    }
  }

  fs::write(&path, filtered_data)?;
  let content = fs::read_to_string(&path)?;
  Ok(content )
}

mod tests {
  #[allow(unused_imports)]
  use super::*;
  #[test]
  fn test_compare_versions() {
    assert_eq!(compare_versions("1.2.3".to_string(), "1.2.3".to_string()), Ordering::Equal);
  }

  #[test]
  fn  test_rm_bom(){
    let path = r"A:\Scoop\buckets\echo\bucket\hdtune.json" ;
     let content = remove_bom_and_control_chars_from_utf8_file(path).unwrap();
     let  content = serde_json::from_str::<serde_json::Value>(&content).unwrap();
  }

}
