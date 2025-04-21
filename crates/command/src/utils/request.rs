use crate::config::get_config_value_no_print;
use crate::utils::utility::write_into_log_file_append_mode;
use anyhow::bail;
use crossterm::style::Stylize;
use git2::{FetchOptions, Progress, ProxyOptions, RemoteCallbacks, Repository};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::get;
use std::fs::{create_dir_all, read_dir, remove_dir, remove_dir_all, remove_file, rename, File};
use std::io::{copy, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use tokio::time;
use zip::ZipArchive;

pub async fn request_download_git_repo(
    url: &str,
    download_path: &str,
) -> Result<String, anyhow::Error> {
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
    let repo_name = archive
        .by_index(0)?
        .name()
        .to_string()
        .trim()
        .replace("/", r"\");
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
        let target_path = entry.to_string_lossy().trim().replace(&repo_name, "");

        let target_path = Path::new(&target_path);
        if entry.is_dir() {
            rename(&entry, &target_path).expect("Failed to rename directory路径错误");
        } else if entry.is_file() {
            rename(&entry, &target_path).expect("Failed to rename directory路径错误");
        }
    }
    remove_dir(current_dir)?;
    Ok("下载成功!!!".dark_green().bold().to_string())
}

pub async fn request_download_git_clone(
    repo_url: &str,
    destination: &str,
) -> Result<String, anyhow::Error> {
    if Path::new(destination).exists() {
        remove_dir_all(destination).expect("Failed to delete directory for bucket ");
    }
    if !Path::new(destination).exists() {
        create_dir_all(destination).expect("Failed to create directory for bucket ");
    }

    println!(
        "正在下载 {} =>  {}",
        repo_url.dark_green().bold(),
        destination.dark_green().bold()
    );
    let output = Command::new("git")
        .arg("clone")
        .arg("--progress")
        .arg(&repo_url)
        .arg(&destination)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    if output.is_err() {
        return request_git_clone_by_git2(repo_url, destination.to_string()).await;
    }
    let mut output = output?;

    let status = output.wait().expect("Failed to wait on child process");

    let mut stderr = output.stderr.unwrap();
    let mut str = String::new();
    stderr.read_to_string(&mut str)?;

    let mut downloaded: f32 = 0.0;
    let size_re = Regex::new(r"(\d+\.\d+\s\w+)")?;
    let speed_regex = Regex::new(r"(\d+\.\d+)\s\w+/s")?;
    let total_size = (|| {
        // 从输出中提取下载大小  KB 或者MB
        return if let Some(size_caps) = size_re.captures(&str) {
            let result = &size_caps[1].trim(); // 提取总大小
            let result = (|| {
                return if result.contains("KiB") {
                    result
                        .replace("KiB", "")
                        .trim()
                        .to_string()
                        .parse::<f32>()
                        .unwrap()
                        * 1024.0
                } else {
                    result
                        .replace("MiB", "")
                        .trim()
                        .to_string()
                        .parse::<f32>()
                        .unwrap()
                        * 1024.0
                        * 1024.0
                };
            })();
            result
        } else {
            println!("Failed to extract total size from output");
            0.0
        };
    })();
    let downloaded_speed = (|| {
        // KB/S 或者 MB/S 的下载速度
        return if let Some(speed_caps) = speed_regex.captures(&str) {
            let result = &speed_caps[0].trim(); // 提取下载速度
                                                // 提取字符串中的数字
                                                // let result = result.replace("KB/s", ""); // &str 存在栈区会被释放 ,需要返回String
                                                // let result = result.replace("MiB/s", "").trim().to_string();
            let result = (|| {
                return if result.contains("KiB/s") {
                    result
                        .replace("KiB/s", "")
                        .trim()
                        .to_string()
                        .parse::<f32>()
                        .unwrap()
                        * 1024.0
                } else {
                    result
                        .replace("MiB/s", "")
                        .trim()
                        .to_string()
                        .parse::<f32>()
                        .unwrap()
                        * 1024.0
                        * 1024.0
                };
            })();

            result
        } else {
            println!("Failed to extract download speed from output");
            0.0
        };
    })();
    // ---
    let pb = ProgressBar::new(total_size as u64);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
    .progress_chars("#>-"));
    while downloaded < total_size {
        let new = (downloaded + downloaded_speed).min(total_size);
        downloaded = new;
        pb.set_position(new as u64);
        // 更新进度条的信息
        //pb.set_message(format!("{:.2} KB / {:.2} MB", downloaded_mb, total_size_mb));

        time::sleep(Duration::from_millis(12)).await;
    }

    pb.finish_with_message("downloaded");

    if status.success() {
        Ok("下载成功!!!".dark_green().bold().to_string())
    } else {
        let mut stderr_bytes = Vec::new();
        stderr
            .read_to_end(&mut stderr_bytes)
            .expect("Failed to read stderr");

        let error_message = String::from_utf8_lossy(&stderr_bytes).to_string();
        println!("克隆失败: {}", error_message);
        bail!(error_message);
    }
}

pub async fn request_git_clone_by_git2(
    repo_url: &str,
    destination: String,
) -> Result<String, anyhow::Error> {
    if Path::new(&destination).exists() {
        remove_dir_all(&destination).expect("Failed to delete directory for bucket ");
    }
    if !Path::new(&destination).exists() {
        create_dir_all(&destination).expect("Failed to create directory for bucket ");
    }
    match Repository::clone(repo_url, &destination) {
        // {:?} 会调用 Display的trait 实现
        Ok(_) => println!("✅ 仓库已克隆到 {}", destination.dark_green().bold()),
        Err(e) => eprintln!("❌ 克隆失败: {}", e),
    }
    Ok("下载成功!!!".dark_green().bold().to_string())
}

pub   fn request_git_clone_by_git2_with_progress(
    repo_url: &str,
    destination: &String,
) -> Result<String, anyhow::Error> {
    if Path::new(destination).exists() {
      let proceed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("目录 {} 已存在，是否删除?", destination))
        .interact()?;
      if proceed {
        remove_dir_all(destination)?;
        println!("已删除目录 {}", destination);
      } else {
        return Ok("保留原目录，操作取消".to_string().dark_green().bold().to_string() );
      }
    }
    if !Path::new(destination).exists() {
        create_dir_all(destination)?
    }
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}%  {msg}")?
            .progress_chars("#>-"),
    );
    let start_time = Instant::now();
    let mut last_received = 0;
    let mut last_time = start_time;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats: Progress| {
    let received_objects = stats.received_objects();
    let total_objects = if stats.total_objects() > 0 {
      stats.total_objects()
    } else {
      stats.indexed_objects()
    };
    if total_objects > 0 {
      let percent = (received_objects as f64 / total_objects as f64) * 100.0;

      // 计算下载速率（单位：对象数/s）
      let now = Instant::now();
      let elapsed = now.duration_since(last_time);
      let delta_objects = received_objects - last_received;

      let speed = if elapsed.as_secs_f64() > 0.0 {
        delta_objects as f64 / elapsed.as_secs_f64()
      } else {
        0.0
      };
      #[cfg(debug_assertions)]
      write_into_log_file_append_mode("git2_clone.txt",
                                      format!("speed {speed:.2}, total_objects {total_objects}, received_objects {received_objects},percent {percent:.2}"));

      pb.set_position(percent as u64);
      pb.set_message(format!(
        "{} / {} objects, {:.2} objs/s",
        received_objects, total_objects, speed
      ));
      pb.tick();
      std::io::stdout().flush().unwrap();
      last_received = received_objects;
      last_time = now;
    }
    true // 继续下载
  });
    let mut proxy_option = ProxyOptions::new();
    let config_proxy = get_config_value_no_print("proxy");
    if !config_proxy.is_empty() {
        let proxy_url = if config_proxy.contains("http://") || config_proxy.contains("https://") {
            config_proxy.clone()
        } else {
            "http://".to_string() + &config_proxy
        };
        proxy_option.url(proxy_url.as_str());
        log::info!("proxy_option {:?}", proxy_url);
    }
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);
    fo.proxy_options(proxy_option);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    match builder.clone(repo_url, Path::new(destination)) {
        Ok(_) => {
            pb.finish_with_message("✅ 成功".to_string());
            println!(
                "✅ 仓库已克隆到 {}",
                destination.to_string().dark_green().bold()
            );
            pb.finish_and_clear();
            Ok("下载成功!!!".dark_green().bold().to_string())
        }
        Err(e) => {
            pb.finish_with_message("❌ 克隆失败！");
            pb.finish_and_clear();
            bail!("❌ 失败: {}", e);
        }
    }
}

#[allow(unused)]
pub async fn download_third_party_buckets() -> Result<String, anyhow::Error> {
    let third_buckets = vec![
        "https://github.com/DoveBoy/Apps",
        "https://github.com/anderlli0053/DEV-tools",
        "https://github.com/kkzzhizhou/scoop-apps",
        "https://github.com/cmontage/scoopbucket",
        "https://github.com/cmontage/scoopbucket-third",
        "https://github.com/chawyehsu/dorado",
        "https://github.com/echoiron/echo-scoop",
        "https://github.com/hoilc/scoop-lemon",
        "https://github.com/lzwme/scoop-proxy-cn",
        "https://github.com/okibcn/ScoopMaster",
        "https://github.com/TheRandomLabs/Scoop-Python",
        "https://github.com/Samiya321/scoop-samiya",
    ];
    let name = vec![
        "DoveBoyApps",
        "DEV-tools",
        "scoop-apps",
        "cmontage",
        "cmontage-third",
        "dorado",
        "echo",
        "lemon",
        "scoop-proxy-cn",
        "okibcn",
        "Python",
        "samiya",
    ];

    for (i, _) in third_buckets.iter().enumerate() {
        let url = third_buckets[i];
        let last_word = &name[i];

        let download_path = "A:/scoop/buckets/".to_string() + &last_word;
        if Path::new(&download_path).exists() {
            continue;
        }

        request_download_git_clone(url, &download_path).await?;
    }

    Ok("下载成功!!!".dark_green().bold().to_string())
}

pub fn get_git_repo_remote_url<P: AsRef<Path>>(repo_path: P) -> Result<String, anyhow::Error> {
    use git2::Repository;
    let repo = Repository::open(repo_path)?;
    for remote in repo.remotes()?.iter() {
        if remote.is_none() {
            continue;
        }
        let remote = remote.unwrap();
        let remote = repo.find_remote(remote);
        if remote.is_err() {
            continue;
        }
        let remote = remote?;
        let url = remote.url().unwrap();
        if url.is_empty() {
            continue;
        }
        return Ok(url.to_string());
    }
    Ok("Not found remote url".to_string())
}
