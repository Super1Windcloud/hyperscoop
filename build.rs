use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;

fn include_aria2() {
  // 压缩 aria2c.exe
  let exe_data = include_bytes!("./resources/aria2c.exe");
  let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
  encoder.write_all(exe_data).unwrap();
  let compressed = encoder.finish().unwrap();

  // 生成解压代码
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let mut file = File::create(Path::new(&out_dir).join("aria2_data.rs")).unwrap();
  writeln!(file, "const ARIA2_DATA: &[u8] = &{:?};", compressed).unwrap();
}


fn main() {
   let  lang =  env::var("LANG").or(env::var("LC_ALL"))
     .or(env::var("LC_CTYPE")).unwrap_or_default();
   let  lang_prefix  = lang.split("_").next().unwrap_or("en");

   println!("cargo:rustc-env=BUILD_SYSTEM_LANG={}" ,lang_prefix);

   if  lang_prefix =="zh" {
      println!("cargo:rustc-cfg=system_lang_zh");
   }
    include_aria2() ;
}


