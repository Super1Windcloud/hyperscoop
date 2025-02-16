use std::cmp::Ordering;
use std::path::Path;
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


mod tests {
  use super::*;

  #[test]
  fn test_compare_versions() {
    assert_eq!(compare_versions("1.2.3".to_string(), "1.2.3".to_string()), Ordering::Equal);
  } 
}