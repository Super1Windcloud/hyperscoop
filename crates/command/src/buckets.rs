use crate::init_hyperscoop;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use crossterm::style::{Color, PrintStyledContent, Stylize};
use regex::Regex;
use serde_json;
use std::fs::{metadata, read_dir, File};
use std::io::BufReader;
use std::process::exit;
use std::time::UNIX_EPOCH;
use log::{error, info, warn};
use reqwest::{get, Error};
#[derive(Debug, Clone)]
pub struct Buckets {
  pub buckets_path: Vec<String>,
  pub buckets_name: Vec<String>,
}
//option 代表可选参数 ,Result 代表Promise的reject 和 resolve
impl Buckets {
  pub async fn add_buckets(
    &self,
    name: &Option<String>,
    url: &Option<String>,
  ) -> Result<(), anyhow::Error> {
    //下载 url 到 bucket_path 下的
    let bucket_name = name
      .clone()
      .unwrap_or_else(|| url.clone().unwrap().split("/").last().unwrap().to_string());
    let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
    let url = url.clone().expect("Failed to initialize hyperscoop"); // 调用 option或Result函数才使用 ?
    if (!url.contains("http://") && !url.contains("https://")) {
      error!("");
      return Err(anyhow!("Invalid URL: {}", url).context("请输入正确的 URL"));
    };

    let result = self.download_bucket(&url, &bucket_name, &hyperscoop.bucket_path).await.expect("Failed to download bucket");
    println!("{}", result);
    Ok(())
  }
  async fn download_bucket(&self, url: &str, bucket_name: &str, bucket_path: &str) -> Result<String, Error> {
    let bucket_path = bucket_path.to_string() + "\\" + bucket_name;
    println!("{:?}  ,  {:?}", bucket_path, url);
    println!("{:?}", bucket_name);
    println!("{} ", "正在下载...... ".dark_green().bold());
    self.request_url(url, &bucket_path).await?;
    return Ok(format!("bucket添加成功 ,\tpath:{}", bucket_path).dark_cyan().bold().to_string());
  }
  async fn request_url(url: &str, bucket_path: &str) -> Result<String, Error> {
    let mut response = get(url).await?;
    let mut file = File::create(bucket_path).expect("Failed to create bucket_path"); // 创建文件
    let mut content = String::new(); // 定义字符串
    response.read_to_string(&mut content);  // 读取响应内容
    file.write_all(content.as_bytes()).expect("Failed to write content"); // 写入文件
  }
}

impl Buckets {
  pub fn display_all_buckets(&self) -> Result<(), anyhow::Error> {
    println!(
      "{:<30}",
      "BucketName\t\t\tUpdated \t\t\tManifest ".dark_blue().bold()
    );
    let (bucket_name, bucket_source, bucket_updated, bucket_manifest) = Self::get_all_buckets();
    let combined_buckets: Vec<(String, String, String, String)> = bucket_name
      .into_iter()
      .zip(bucket_source.into_iter())
      .zip(bucket_updated.into_iter())
      .zip(bucket_manifest.into_iter())
      .map(|(((name, source), updated), manifest)| (name, source, updated, manifest))
      .collect();
    for (name, source, updated, manifest) in combined_buckets.iter() {
      println!("{:<30} \t{:<30}\t{:<30}", name, updated, manifest);
    }
    return Ok(());
  }
  fn get_all_buckets() -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let buckets = Buckets::new();
    let bucket_name = buckets.buckets_name.clone();
    let bucket_source = buckets.buckets_path.clone();
    let bucket_updated = Self::get_updated_time(&bucket_source);
    let bucket_manifest = Self::get_manifest_version(&bucket_source);

    return (bucket_name, bucket_source, bucket_updated, bucket_manifest);
  }

  fn get_updated_time(bucket_source: &Vec<String>) -> Vec<String> {
    let mut bucket_updated: Vec<String> = Vec::new();

    for source in bucket_source {
      let path = source.to_string() + "\\bucket";
      let metadata = metadata(&path).expect("Failed to get metadata");
      let modified_time = metadata.modified().expect("Failed to get modified time");
      // 将修改时间转换为自 UNIX_EPOCH 以来的时间戳
      let duration_since_epoch = modified_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
      let updated_time = UNIX_EPOCH + duration_since_epoch; // 这里得到的是一个`SystemTime`
      let updated_time_utc: DateTime<Utc> = updated_time.into(); // 转换为 `DateTime<Utc>`
      let updated_time_formatted = updated_time_utc.format("%Y-%m-%d %H:%M:%S").to_string();
      bucket_updated.push(updated_time_formatted.trim_matches('"').into());
    }
    return bucket_updated;
  }

  fn get_manifest_version(path: &Vec<String>) -> Vec<String> {
    let mut bucket_manifest: Vec<String> = Vec::new();
    // 获取目录的子文件个数
    for source in path {
      let source = source.to_string() + "\\bucket";
      let mut count = 0;
      count = read_dir(source).expect("Failed to read directory").count(); // 这里得到的是一个`u64`
      bucket_manifest.push(count.to_string());
    }

    return bucket_manifest;
  }
}

impl Buckets {
  pub fn get_bucket_self(&self) -> Result<(Vec<String>, Vec<String>), anyhow::Error> {
    let bucket = Buckets::new();
    Ok((bucket.buckets_path, bucket.buckets_name))
  }
  pub fn new() -> Buckets {
    let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
    let bucket_path = hyperscoop.bucket_path.clone();
    // 遍历 bucket_path 下的所有文件夹，并将文件夹名加入 buckets_path
    let buckets_path: Vec<String> = std::fs::read_dir(&bucket_path)
      .expect("Failed to read bucket_path")
      .filter_map(|e| e.ok())
      .filter(|e| e.path().is_dir())
      .map(|e| e.path().to_str().unwrap().to_string())
      .collect();
    let buckets_name: Vec<String> = buckets_path
      .iter()
      .map(|e| e.split("\\").last().unwrap().to_string())
      .collect();
    Buckets {
      buckets_path: buckets_path,
      buckets_name: buckets_name,
    }
  }

  pub fn get_bucket_known(&self) -> Result<(Vec<String>, Vec<String>), anyhow::Error> {
    let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
    let known_bucket_path = hyperscoop.apps_path.clone() + "\\scoop\\current\\buckets.json";
    let file_buffer = File::open(&known_bucket_path).expect("Failed to open known_bucket_path");
    let reader_buffer = BufReader::new(file_buffer);
    let content: serde_json::Value = serde_json::from_reader(reader_buffer)?;
    let mut known_name: Vec<String> = Vec::new();
    let mut known_source: Vec<String> = Vec::new();
    let re = Regex::new(r#""(https?://[^\s]+)""#).unwrap();
    for bucket in content.as_object().unwrap() {
      let name = bucket.0.to_string();
      let source = bucket.1.to_string();
      if let Some(captures) = re.captures(&source) {
        let url = &captures[1]; // 提取捕获的第一个组，即 URL
        known_source.push(url.to_string());
      } else {
        println!("未找到 URL");
      };
      known_name.push(name);
    }
    return Ok((known_name, known_source));
  }

  //iter() 用于获取集合中元素的不可变引用，允许直接访问元素。
  // enumerate() 用于在遍历时提供每个元素的索引，通常与其他迭代器方法组合使用。
  pub fn display_known_buckets(&self) -> Result<(), anyhow::Error> {
    let (known_name, known_source) = self.get_bucket_known().unwrap();
    println!("{}", "BucketName\t\t\t | SourceUrl  ".dark_cyan().bold());
    for (name, source) in known_name.iter().zip(known_source.iter()) {
      println!("{:<15}\t\t\t | {}", name, source);
    }
    return Ok(());
  }
}
