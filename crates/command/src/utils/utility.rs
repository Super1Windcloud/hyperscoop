use std::cmp::Ordering;

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