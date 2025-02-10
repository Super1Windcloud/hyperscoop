use std::cmp::Ordering;

pub mod request;
pub mod  progrees_bar ;
pub mod detect_encoding;
pub mod repair_dirty_json;
pub mod safe_check;
pub mod get_file_or_dir_metadata;
pub mod  system ;
pub mod  git; 
pub mod  invoke_hook_script ;
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



