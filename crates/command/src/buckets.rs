use anyhow::anyhow;
use chrono::{DateTime, Utc};
use crossterm::style::{Stylize};
use regex::Regex;
use serde_json;
use std::fs::{create_dir_all, metadata, read_dir, remove_dir, remove_dir_all, remove_file, rename, File};
use std::io;
use std::io::{BufReader, Write};
use std::path::{Path};
// use std::process::exit;
use std::time::UNIX_EPOCH;
use log::{error};
use reqwest::{get};
use zip::read::ZipArchive;
use crate::utils::request::{get_git_repo_remote_url, request_download_git_clone};
use crate::init_hyperscoop;

#[derive(Debug, Clone)]
pub struct Buckets {
  pub buckets_path: Vec<String>,
  pub buckets_name: Vec<String>,
}
pub fn get_buckets_path() -> Result<Vec<String>, anyhow::Error> {
  let bucket = Buckets::new();
  let buckets_path = bucket.buckets_path;
  return Ok(buckets_path);
}

pub fn get_buckets_name() -> Result<Vec<String>, anyhow::Error> {
  let bucket = Buckets::new();
  let buckets_name = bucket.buckets_name;
  return Ok(buckets_name);
}
impl Buckets {
  //参数传递尽量以借用为主，避免拷贝大量数据
  pub async fn rm_buckets(&self, name: &String) -> Result<(), anyhow::Error> {
    let (bucket_paths, buckets_name) = self.get_bucket_self()?;
    for bucket_name in buckets_name {
      if &bucket_name == name {
        for bucket_path in &bucket_paths {
          if bucket_path.ends_with(name) {
            let delete_path = Path::new(bucket_path);
            self.delete_dir_recursively(&delete_path).expect("Failed to remove directory");
            println!("{}", "删除成功".dark_red().bold().to_string());
            return Ok(());
          }
        }
      }
    }
    error!("");
    Err(anyhow!("bucket not found").context("没有这个名字的bucket"))
  }
  fn delete_dir_recursively(&self, bucket_path: &Path) -> Result<(), anyhow::Error> {
    println!("正在删除目录 : {:?}", bucket_path);
    for entry in read_dir(bucket_path)? {
      let path = entry?.path();

      if path.is_dir() {
        remove_dir_all(&path)?;
      } else {
        remove_file(&path)?
      }
    }
    remove_dir(bucket_path)?;
    Ok(())
  }
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
    if !url.contains("http://") && !url.contains("https://") {
      error!("");
      return Err(anyhow!("Invalid URL: {}", url).context("请输入正确的 URL"));
    };

    let result = self.download_bucket(&url, &bucket_name, &hyperscoop.bucket_path).await.expect("Failed to download bucket");
    println!("{}", result);
    Ok(())
  }
  pub async fn download_bucket(&self, url: &str, bucket_name: &str, bucket_path: &str) -> Result<String, anyhow::Error> {
    let bucket_path = bucket_path.to_string() + "\\" + bucket_name;
    // println!("{:?}  ,  {:?}", bucket_path, url);
    // println!("{:?}", bucket_name);
    println!("{} ", "正在下载...... ".dark_green().bold());

    // let result = download_third_party_buckets().await?;
    let result = request_download_git_clone(&url, &bucket_path).await?;
    println!("{} ", result);
    return Ok(format!("bucket添加成功\t{}", bucket_path).dark_cyan().bold().to_string());
  }
  pub fn check_file_ishave_content(&self, bucket_path: &str) -> Result<(), anyhow::Error> {
    //检查目录是否包含文件
    if !Path::new(bucket_path).read_dir()?.next().is_none() {
      return Err(anyhow!("当前目录已经存在文件，请先清空目录或创建新目录: {}", bucket_path));
    }
    Ok(())
  }
  pub async fn request_url(&self, url: &str, bucket_path: &str) -> Result<String, anyhow::Error> {
    self.check_file_ishave_content(bucket_path)?;
    let mut url = url.to_string();
    let mut branch_flag = "-master".to_string();
    if url.contains(".git") {
      //  let 可以进行变量遮蔽重新赋值
      url = url.replace(".git", "");
    }
    // 将 repo_url 转换为 ZIP 下载链接 ,下载github仓库的zip压缩包
    let zip_url = format!("{}/archive/refs/heads/master.zip", url);
    let backup_zip_url1 = format!("{}/archive/refs/heads/main.zip", url);
    let backup_zip_url2 = format!("{}/archive/refs/heads/dev.zip", url);
    let mut response = get(zip_url).await?;
    if !response.status().is_success() {
      response = get(backup_zip_url1).await?;
      branch_flag = "-main".to_string();
      if !response.status().is_success() {
        response = get(backup_zip_url2).await?;
        branch_flag = "-dev".to_string();
      }
    }
    //url 是git仓库地址，bucket_path 是下载到本地的路径
    // 创建一个文件用于存储 ZIP 数据
    let zip_path = Path::new(bucket_path).join("repo.zip");
    if !Path::new(bucket_path).exists() {
      create_dir_all(&bucket_path).expect("Failed to create directory for bucket ");
    }
    let mut file = File::create(&zip_path)?;
    // 将下载的数据写入文件
    let content = response.bytes().await?;
    file.write_all(&content)?;

    // 解压 ZIP 文件
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let repo_name = archive.by_index(0)?.name().to_string()
      .trim().replace("/", r"\");
    // 创建目标文件夹
    let dest = Path::new(bucket_path);
    create_dir_all(&dest)?;

    // 解压文件到目标文件夹
    for i in 0..archive.len() {
      let mut file = archive.by_index(i)?;
      let outpath = dest.join(file.name());
      if file.name().ends_with('/') {
        // 如果是文件夹，创建目录
        create_dir_all(&outpath)?;
      } else {
        // 如果是文件，写入文件
        let mut outfile = File::create(&outpath)?;
        io::copy(&mut file, &mut outfile)?;
      }
    }
    // 删除 ZIP 文件
    remove_file(&zip_path)?;
    let last_url = url.split("/").last().unwrap().to_string();
    let current_dir = dest.join(last_url + &branch_flag);
    // println!("{:?} ", current_dir);
    // println!("{:?} ", dest);
    //遍历源目录中的所有项
    for entry in read_dir(&current_dir)? {
      let error_message = format!("无法读取目录 {}", current_dir.clone().display());
      let path = entry.expect(error_message.as_str()).path();
      let entry: &Path = path.as_ref();
      let target_path = entry.to_string_lossy()
        .trim().replace(&repo_name, "");
      //println!("{}", repo_name);   将/换成\

      // println!(" entry {:?} ", entry.to_string_lossy());
      //   打印出来的是\\ ,但是在原始字符串中是\, 所以在替换的时候只需把\替换成""即可
      // println!("target {:?} ", target_path);
      let target_path = Path::new(&target_path);
      if entry.is_dir() {
        rename(&entry, &target_path).expect("Failed to rename directory路径错误");
      } else if entry.is_file() {
        rename(&entry, &target_path).expect("Failed to rename directory路径错误");
      }
    }
    remove_dir(current_dir)?;
    Ok("下载成功!!! ".dark_green().bold().to_string())
  }
}


impl Buckets {
  pub fn display_all_buckets(&self) -> Result<(), anyhow::Error> {

    let (bucket_name, bucket_source, bucket_updated, bucket_manifest) = Self::get_all_buckets();
    let combined_buckets: Vec<(String, String, String, String)> = bucket_name
      .into_iter()
      .zip(bucket_source.into_iter())
      .zip(bucket_updated.into_iter())
      .zip(bucket_manifest.into_iter())
      .map(|(((name, source), updated), manifest)| (name, source, updated, manifest))
      .collect();
    let max_name_len  = combined_buckets
     .iter()
     .map(|e| e.0.len())
     .max()
     .unwrap_or(0);
  let  max_manifest_len = combined_buckets
     .iter()
     .map(|e| e.3.len())
     .max()
     .unwrap_or(0);
    let max_source_len = combined_buckets
     .iter()
     .map(|e| e.1.len())
     .max()
     .unwrap_or(0);
    let max_updated_len = combined_buckets
    .iter()
    .map(|e| e.2.len())
    .max()
    .unwrap_or(0);

    println!("{:<max_name_len$} {:<max_source_len$} {:<max_updated_len$} {:<max_manifest_len$}",
             "BucketName\t\t".dark_cyan().bold(), "SourceUrl\t\t\t\t\t\t".dark_cyan().bold(),
             "UpdatedTime\t\t".dark_cyan().bold(), "     Manifests".dark_cyan().bold(),
             max_name_len=max_name_len+10, max_source_len=max_source_len,
             max_updated_len=max_updated_len, max_manifest_len=max_manifest_len   );
    for (name, source, updated, manifest) in combined_buckets.iter() {
      println!("{:<max_name_len$} {:<max_source_len$} {:<max_updated_len$} {:<max_manifest_len$}",
               name, source , updated, manifest,
               max_name_len=max_name_len+10, max_source_len=max_source_len,
               max_updated_len=max_updated_len+10 , max_manifest_len=max_manifest_len  );
    }

    Ok(())
  }
  fn get_all_buckets() -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let bucket_name =  get_buckets_name().unwrap();
    let bucket_source_url  = Self::get_bucket_source_url ();
    let bucket_source =  get_buckets_path().unwrap();
    let bucket_updated = Self::get_updated_time(&bucket_source);
    let bucket_manifest = Self::get_manifest_version(&bucket_source);

    return (bucket_name, bucket_source_url, bucket_updated, bucket_manifest);
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

      let count = read_dir(source).expect("Failed to read directory").count(); // 这里得到的是一个`u64`
      bucket_manifest.push(count.to_string());
    }

    return bucket_manifest;
  }

  fn get_bucket_source_url() -> Vec<String>  {
    let bucket_path  = get_buckets_path().unwrap();
    let buckets_path = bucket_path.iter().map(
      |path| get_git_repo_remote_url(path).unwrap()
    ).collect::<Vec<String>>();


    buckets_path
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
