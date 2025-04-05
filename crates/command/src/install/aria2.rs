use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn  get_aria2_log_path   () ->  PathBuf  {
    let cwd = env::current_dir().unwrap(); 
     let  log  = cwd.parent().unwrap().parent().unwrap() .join("log");
    let log_path = Path::new(&log).join("aria2.log");
    if log_path.exists() {
        log_path 
    } else {
        let _ = std::fs::create_dir_all(&log_path); // 创建父目录 
        File::create(&log_path).unwrap();
        log_path 
    }
}
pub fn include_aria2() {
    let exe_data = include_bytes!("../../../../resources/aria2c.exe");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(exe_data).unwrap();
    let compressed = encoder.finish().unwrap();

    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| {
        let fallback = Path::new("target").join("_aria2");
        std::fs::create_dir_all(&fallback).unwrap();
        fallback.to_str().unwrap().to_string()
    });
    let data_path = Path::new(&out_dir).join("aria2_data.rs");
    let mut file = File::create(&data_path).unwrap();
    writeln!(file, "const ARIA2_DATA: &[u8] = &{:?};", compressed).unwrap();

    let log_path = get_aria2_log_path();
    let mut log = File::create(&log_path).unwrap();
    env::set_var("ARIA2_DATA_PATH", data_path.to_str().unwrap());
    let env =  env::var("ARIA2_DATA_PATH")
        .unwrap_or("No ARIA2_DATA_PATH env variable set.".to_string());
    writeln!(
        log,
        "ARIA2_DATA_PATH : {}\n log_path :{}\n  ENV :{env}",
        data_path.display(),
        log_path.display()
    )
    .unwrap();
}

pub fn get_temp_aria2_path() -> String {
    let temp_dir = std::env::temp_dir();
    let exe_path = temp_dir.join("aria2c.exe");
    exe_path.to_str().unwrap().to_string()
}
pub fn extract_aria2() -> anyhow::Result<()> {
    let embeded_path = std::env::var("ARIA2_DATA_PATH")?;
    let embeded_path = Path::new(&embeded_path);
    let embeded_file = File::open(embeded_path)?;
    let mut decoder = GzDecoder::new(embeded_file);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;

    let temp_dir = std::env::temp_dir();
    let exe_path = temp_dir.join("aria2c.exe");

    let mut file = File::create(&exe_path)?;
    file.write_all(&decompressed_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use std::env;

    #[test]
    fn aria2_log() { 
        let log_path = get_aria2_log_path();
    }
    #[test]
    fn test_aria2() {
        let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| {
            let cwd = env::current_dir().unwrap();
            let fallback = Path::new("target").join("_aria2");
            std::fs::create_dir_all(&fallback).unwrap();
            fallback.to_str().unwrap().to_string()
        });

        let data_path = Path::new(&out_dir).join("aria2_data.rs");
        println!("data path: {}", &data_path.display());
        // extract_aria2().unwrap();
        let dara_path = data_path.to_str().unwrap();
        std::env::set_var("ARIA2_DATA_PATH", data_path);
        let exe_path = get_temp_aria2_path();
        if Path::new(&exe_path).exists() {
            println!("aria2c.exe 解压成功");
        }
    }
}
