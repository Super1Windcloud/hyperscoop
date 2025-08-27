use std::fs::read_dir;
use std::path::PathBuf;

pub fn is_directory_empty(path: &PathBuf) -> bool {
    let entries = read_dir(path);

    match entries {
        Ok(mut dir_entries) => match dir_entries.next() {
            Some(_) => false,
            None => true,
        },
        Err(_) => {
            eprintln!("无法读取目录: {}", path.display());
            false
        }
    }
}
