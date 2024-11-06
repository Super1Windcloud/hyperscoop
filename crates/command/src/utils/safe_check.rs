use std::fs::{read_dir};
use std::path::PathBuf;

pub fn is_directory_empty(path: &PathBuf) -> bool {
  let entries = read_dir(path);

  match entries {
    Ok(mut dir_entries) => {
      // 尝试读取第一个条目
      match dir_entries.next() {
        Some(_) => false, // 目录中有至少一个条目
        None => true,     // 目录为空
      }
    }
    Err(_) => {
      // 处理错误，例如路径不存在或不可读取
      eprintln!("无法读取目录: {}", path.display());
      false // 在出错的情况下返回 false
    }
  }
}
