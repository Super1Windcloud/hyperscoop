use std::fs::{create_dir_all, read_dir, remove_dir, remove_dir_all, remove_file, rename, File};
use std::io;
use std::io::{copy, Write};
use std::path::Path;
use std::process::{exit, Command};
use std::thread::ThreadId;
use crossterm::style::Stylize;
use reqwest::get;
use zip::ZipArchive;

pub async fn request_download_git_repo(url: &str, download_path: &str) -> Result<String, anyhow::Error> {
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
  let zip_path = Path::new(download_path).join("repo.zip");
  if !Path::new(download_path).exists() {
    create_dir_all(&download_path).expect("Failed to create directory for bucket ");
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
  let dest = Path::new(download_path);
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
      copy(&mut file, &mut outfile)?;
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
    //下载 zip
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


pub async fn request_download_git_clone(repo_url: &str, destination: &str) -> Result<String, io::Error> {
  // 创建一个新的命令

  if Path::new(destination).exists() {
    remove_dir_all(destination).expect("Failed to delete directory for bucket ");
  }
  if !Path::new(destination).exists() {
    create_dir_all(destination).expect("Failed to create directory for bucket ");
  }
  println!("正在下载 {} 到 {}", repo_url, destination);
  let output = Command::new("git")
    .arg("clone")
    .arg(repo_url)
    .arg(destination)
    .output()?;

  // 检查命令是否成功执行
  if output.status.success() {
    Ok("下载成功!!! ".dark_green().bold().to_string())
  } else {
    let error_message = String::from_utf8_lossy(&output.stderr);
    println!("克隆失败: {}", error_message);
    Err(io::Error::new(io::ErrorKind::Other, "克隆失败"))
  }
}
pub async fn download_third_party_buckets() -> Result<String, anyhow::Error> {
  let third_buckets = vec!["https://github.com/DoveBoy/Apps",
                           "https://github.com/anderlli0053/DEV-tools", "https://github.com/kkzzhizhou/scoop-apps",
                           "https://github.com/cmontage/scoopbucket", "https://github.com/cmontage/scoopbucket-third",
                           "https://github.com/chawyehsu/dorado", "https://github.com/echoiron/echo-scoop",
                           "https://github.com/hoilc/scoop-lemon", "https://github.com/lzwme/scoop-proxy-cn",
                           "https://github.com/okibcn/ScoopMaster",
                           "https://github.com/TheRandomLabs/Scoop-Python",
                           "https://github.com/Samiya321/scoop-samiya"
  ];
  let name = vec!["DoveBoyApps", "DEV-tools", "scoop-apps", "cmontage", "cmontage-third", "dorado", "echo",
                  "lemon", "scoop-proxy-cn", "okibcn", "Python", "samiya"];

  for (i, _) in third_buckets.iter().enumerate() {
    let url = third_buckets[i];
    let last_word = &name[i];

    let download_path = "A:/scoop/buckets/".to_string() + &last_word;
    if Path::new(&download_path).exists() {
      continue;
    }

    request_download_git_clone(url, &download_path).await?;
  }

  Ok(("下载成功!!! ".dark_green().bold().to_string()))
}
