use anyhow::{Context, bail};
use crossterm::style::Stylize;
use encoding::all::{GBK, UTF_8, UTF_16BE, UTF_16LE};
use encoding::label::encoding_from_whatwg_label;
use encoding::{DecoderTrap, Encoding};
use log::error;
use serde_json::from_str;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::str::from_utf8;

use crate::manifest::search_manifest::SearchManifest;

pub fn transform_to_only_version_manifest(path: &Path) -> Result<SearchManifest, anyhow::Error> {
    let file = File::open(path).context("Could not open file at line 17")?;
    let mut content = String::new();
    let mut reader = BufReader::new(file);
    reader
        .read_to_string(&mut content)
        .expect("读取only_version失败");
    let trimmed = content.trim_start_matches('\u{feff}');
    // 去除控制字符
    let cleaned = trimmed
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();
    let result = from_str(&cleaned);

    match result {
        Ok(manifest) => Ok(manifest),
        Err(e) => {
            bail!("解析only_version失败: {}", e);
        }
    }
}

pub fn judge_is_valid_utf8_exclude_bom(path: &Path) -> bool {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut content = Vec::new();

    // 读取文件内容到字节数组
    reader.read_to_end(&mut content).unwrap();
    // 尝试将字节内容解码为 UTF-8
    let result = from_utf8(&content).map(|_| true).map_err(|_| false).ok();

    if let Some(false) = result {
        //transform_file_to_utf8(path).expect("转换失败");
        error!("路径{}解码失败 ", &path.to_string_lossy());
        return false;
    }
    true
}

pub fn judge_is_gbk(path: &Path) -> bool {
    // 使用 encoding 库的功能来检测编码
    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut best_guess = None;
    let options = ["GBK", "GB18030", "GB2312"]; // 可能的 GBK 相关编码

    for encoding_str in options {
        if let Some(encoding) = encoding_from_whatwg_label(encoding_str) {
            let text = encoding
                .decode(&buffer, DecoderTrap::Strict)
                .expect("检测解码失败");
            // 检查解码结果是否有效
            if text.chars().all(|c| c.is_ascii() || !c.is_control()) {
                best_guess = Some(encoding_str);
                break;
            }
        }
    }
    match best_guess {
        None => false,
        Some(encoding_mode) => {
            // println!(" current encoding mode is {}", encoding_mode);
            if encoding_mode == "GBK" {
                // println!("{} {}", "GBK detected in path.".dark_green().bold(), path.display().to_string().dark_green().bold());
                true
            } else {
                false
            }
        }
    }
}

pub fn judge_utf8_is_having_bom(path: &Path) -> bool {
    let mut file = File::open(path).unwrap();
    let mut buffer = [0; 3]; // BOM 最多三个字节
    file.read(&mut buffer).unwrap();

    if buffer == [0xEF, 0xBB, 0xBF] {
        println!(
            "{} {}",
            "UTF-8 with BOM detected in path.".dark_green().bold(),
            path.display().to_string().dark_green().bold()
        );
        true
    } else {
        //  println!("UTF-8 is not having BOM ");
        false
    }
}

pub fn detect_encoding<R: Read>(reader: &mut R) -> Option<&'static dyn Encoding> {
    let mut buf = [0; 3];
    if reader.read(&mut buf).is_ok() {
        // 检测 BOM
        if &buf[..] == [0xEF, 0xBB, 0xBF] {
            return Some(UTF_8);
        } else if &buf[..2] == [0xFF, 0xFE] {
            return Some(UTF_16LE);
        } else if &buf[..2] == [0xFE, 0xFF] {
            return Some(UTF_16BE);
        }
    }
    // 没有 BOM，假设是 GBK
    None
}

pub fn transform_to_search_manifest_object(path: &Path) -> Result<SearchManifest, anyhow::Error> {
    if !path.exists() {
        return Ok(SearchManifest { version: None });
    }
    let json_str = read_str_from_json_file(path).expect("读取JSON文件失败");
    if json_str.is_empty() {
        return Ok(SearchManifest { version: None });
    }
    let value: SearchManifest = from_str(&json_str.trim())
        .map_err(|e| {
            return anyhow::anyhow!("转为 serde_json::Value 失败: {}", e);
        })
        .unwrap_or_default();
    Ok(value)
}

pub fn read_str_from_json_file(path: &Path) -> Result<String, anyhow::Error> {
    let result_str = transform_file_to_utf8(path).expect("转换失败");
    // 去除控制字符
    let result_str = result_str
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();
    Ok(result_str)
}

///  将所有非 UTF-8 编码的文件转换为 UTF-8 编码的文件并且返回转换后的字符串
pub fn transform_file_to_utf8(path: &Path) -> Result<String, anyhow::Error> {
    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在"));
    }
    if path.is_file() {
        // 是否是有效的 UTF-8 文件
        if !judge_is_valid_utf8_exclude_bom(path) {
            if judge_is_gbk(path) {
                convert_gbk_to_utf8(path).expect("转换失败");
            } else {
                return Err(anyhow::anyhow!("不是有效的 UTF-8 文件"));
            }
        }
        if judge_utf8_is_having_bom(path) {
            convert_utf8bom_to_utf8(path).expect("转换失败");
        }
    }
    let mut json_str = String::new();
    // 读取文件内容到字符串中
    let file = File::open(path).expect("打开文件失败");
    let mut reader = file;
    match reader.read_to_string(&mut json_str) {
        Ok(_) => None::<usize>,
        Err(_) => {
            println!("转换GBK编码{}", path.display());
            convert_gbk_to_utf8(path).expect("转换失败");
            None
        }
    };
    Ok(json_str)
}

/// 将文件转换为 UTF-8 编码重新写入文件
pub fn convert_gbk_to_utf8(file_path: &Path) -> Result<(), anyhow::Error> {
    // if !judge_is_gbk(file_path) { return Err(anyhow::anyhow!("不是 GBK 编码文件")); }
    // 读取原始文件的内容
    let file = File::open(file_path).context("Could not open file at line 187")?;
    let mut reader = BufReader::new(file);
    let mut content = Vec::new();
    // read_to_end() 方法读取整个文件的内容到缓冲区字节数组中
    reader
        .read_to_end(&mut content)
        .context("Failed to read file content to buffer at line 192")?;
    let content = GBK.decode(&content, DecoderTrap::Strict);
    let utf8_content = &content.expect("转换失败");
    // print!("{}", utf8_content);

    // let utf8_content = UTF_8.encode(&utf8_content, EncoderTrap::Strict).expect("转换失败");
    let mut output_file = OpenOptions::new()
        .write(true)
        .truncate(true) // 清空文件内容
        .open(file_path)
        .context("Could not open file at line 202")?;
    output_file
        .write_all(utf8_content.as_bytes())
        .context("Failed to write file content from buffer at line 204")?;
    println!("GBK file converted to UTF-8 successfully.");
    Ok(())
}

/// 去掉 UTF-8 编码文件的 BOM 并转换为 UTF-8 编码
pub fn convert_utf8bom_to_utf8(file_path: &Path) -> Result<(), anyhow::Error> {
    if !judge_utf8_is_having_bom(file_path) {
        return Err(anyhow::anyhow!("不是 UTF-8-BOM编码文件"));
    }
    // 读取原始文件的内容
    let file = File::open(file_path).context("Failed to open utf8bom file at line 216")?;
    let mut reader = BufReader::new(file);
    let mut content = Vec::new();
    // read_to_end() 方法读取整个文件的内容到缓冲区字节数组中
    // read_to_string() 方法读取整个文件的内容到字符串中
    reader.read_to_end(content.as_mut()).expect("读取文件失败");
    // 检查并移除 UTF-8 BOM
    const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];
    if content.starts_with(&BOM) {
        // 从向量中批量删除指定范围，将所有已删除的元素作为迭代器返回。
        // 如果迭代器在完全使用之前被删除，它将删除剩余的已删除元素。
        content.drain(0..BOM.len());
    }
    // 转换为 UTF-8 编码
    let utf8_content = UTF_8
        .decode(&content, DecoderTrap::Strict)
        .expect("转换失败");

    let mut output_file = OpenOptions::new()
        .write(true)
        .truncate(true) // 清空文件内容
        .open(file_path)
        .context("Failed to open utf8bom file at line 238")?;
    output_file
        .write_all(&utf8_content.as_bytes())
        .expect("写入文件失败,文件可能被占用");
    println!("UTF-8-BOM file converted to UTF-8 successfully.");
    Ok(())
}
