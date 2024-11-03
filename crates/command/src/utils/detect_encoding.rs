use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use encoding_rs::{DecoderResult, Encoding, UTF_8, UTF_16LE, UTF_16BE, GBK};

pub fn detect_encoding<R: Read>(reader: &mut R) -> Option<&'static Encoding> {
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

pub fn read_str_from_json(path: &Path) -> Result<String, anyhow::Error> {
  let file = File::open(path)?;
  let mut reader = file;
  let mut buffer = Vec::new();
  reader.read_to_end(&mut buffer)?;
  let json_str = String::from_utf8(buffer).expect("Invalid UTF-8解析失败");
  Ok(json_str)
}
