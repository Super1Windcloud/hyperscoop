use crate::init_env::{
    get_apps_path, get_apps_path_global, get_buckets_root_dir_path,
    get_buckets_root_dir_path_global,
};
use crate::utils::request::{get_git_repo_remote_url, request_git_clone_by_git2_with_progress};
use anyhow::{anyhow, bail, Context};
use chrono::{DateTime, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::{UTF8_BORDERS_ONLY};
use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use regex::Regex;
use reqwest::get;
use serde_json;
use std::fs::{
    create_dir_all, metadata, read_dir, remove_dir, remove_dir_all, remove_file, rename, File,
};
use std::io;
use std::io::{BufReader, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;
use zip::read::ZipArchive;

#[derive(Debug, Clone)]
pub struct Buckets {
    pub buckets_path: Vec<String>,
    pub buckets_name: Vec<String>,
    pub global_buckets_paths: Vec<String>,
    pub global_buckets_names: Vec<String>,
}

impl Buckets {
    pub fn is_valid_url(&self, url: &String) -> bool {
        let re = Regex::new(r"^(http|https)://[a-zA-Z0-9-.]+\.[a-zA-Z]{2,}(/\S*)?$").unwrap();
        re.is_match(url)
    }
}

pub fn get_buckets_path() -> Result<Vec<String>, anyhow::Error> {
    let bucket = Buckets::new()?;
    let buckets_path = bucket.buckets_path;
    Ok(buckets_path)
}
pub fn get_global_all_buckets_dir() -> anyhow::Result<Vec<String>> {
    let bucket = Buckets::new()?;
    let buckets_path = bucket.global_buckets_paths;
    Ok(buckets_path)
}
pub fn get_global_all_buckets_name() -> anyhow::Result<Vec<String>> {
    let bucket = Buckets::new()?;
    let buckets_name = bucket.global_buckets_names;
    Ok(buckets_name)
}
pub fn get_buckets_name() -> Result<Vec<String>, anyhow::Error> {
    let bucket = Buckets::new()?;
    let buckets_name = bucket.buckets_name;
    Ok(buckets_name)
}
impl Buckets {
    pub fn rm_buckets(&self, name: &String, is_global: bool) -> Result<(), anyhow::Error> {
        let (bucket_paths, buckets_names) = if is_global {
            self.get_global_bucket_self()?
        } else {
            self.get_bucket_self()?
        };
        if buckets_names.is_empty() || bucket_paths.is_empty() {
            bail!("buckets dir not exist or not dir")
        }
        for bucket_name in buckets_names {
            if &bucket_name == name {
                for bucket_path in &bucket_paths {
                    if bucket_path.ends_with(name) {
                        let delete_path = Path::new(bucket_path);
                        self.delete_dir_recursively(&delete_path)
                            .expect("Failed to remove directory");
                        println!("{}", "删除成功".dark_red().bold().to_string());
                        return Ok(());
                    }
                }
            }
        }
        Err(anyhow!("bucket not found").context("没有这个名字的bucket"))
    }
    fn delete_dir_recursively(&self, bucket_path: &Path) -> Result<(), anyhow::Error> {
        println!(
            "{}{}",
            "正在删除目录 : ".to_string().dark_blue().bold(),
            &bucket_path.display().to_string().dark_green().bold()
        );
        remove_dir_all(bucket_path)
          .context(format!("Failed to remove directory: {} at line 91", bucket_path.display()))?;
        Ok(())
    }
}

impl Buckets {
    pub fn add_buckets(
        &self,
        name: &Option<String>,
        url: &Option<String>,
        is_global: bool,
    ) -> Result<(), anyhow::Error> {
        let bucket_name = name
            .clone()
            .unwrap_or_else(|| url.clone().unwrap().split("/").last().unwrap().to_string());

        let url = if url.is_some() {
            url.clone().unwrap()
        } else {
            bail!("URL 不能为空")
        };
        check_name_is_valid(&bucket_name)?;
        if !url.contains("http://") && !url.contains("https://") {
            return Err(anyhow!("Invalid URL: {}", url).context("请输入正确的 URL"));
        };
        let bucket_root_dir = if is_global {
            get_buckets_root_dir_path_global()
        } else {
            get_buckets_root_dir_path()
        };
        if !Path::new(&bucket_root_dir).exists() || !Path::new(&bucket_root_dir).is_dir() {
            bail!("bucket root dir not exist or not dir")
        }
        let result = self
            .download_bucket(&url, &bucket_name, &bucket_root_dir)
            .expect("Failed to download bucket");
        println!("{}", result);
        Ok(())
    }

    pub fn download_bucket(
        &self,
        url: &str,
        bucket_name: &str,
        bucket_path: &str,
    ) -> Result<String, anyhow::Error> {
        let bucket_path = bucket_path.to_string() + "\\" + bucket_name;
        println!("{} ", "开始下载...... ".dark_green().bold());
        let result = request_git_clone_by_git2_with_progress(url, &bucket_path)?;
        Ok(result)
    }
    pub fn check_file_ishave_content(&self, bucket_path: &str) -> Result<(), anyhow::Error> {
        // 检查目录是否包含文件
        if !Path::new(bucket_path).read_dir()?.next().is_none() {
            return Err(anyhow!(
                "当前目录已经存在文件，请先清空目录或创建新目录: {}",
                bucket_path
            ));
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
        // 创建一个文件用于存储 ZIP 数据
        let zip_path = Path::new(bucket_path).join("repo.zip");
        if !Path::new(bucket_path).exists() {
            create_dir_all(&bucket_path)?
        }
        let mut file = File::create(&zip_path)?;
        // 将下载的数据写入文件
        let content = response.bytes().await?;
        file.write_all(&content)?;

        let file = File::open(&zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        let repo_name = archive
            .by_index(0)?
            .name()
            .to_string()
            .trim()
            .replace("/", r"\");
        let dest = Path::new(bucket_path);
        create_dir_all(&dest)?;

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

        for entry in read_dir(&current_dir)? {
            let error_message = format!("无法读取目录 {}", current_dir.clone().display());
            let path = entry.expect(error_message.as_str()).path();
            let entry: &Path = path.as_ref();
            let target_path = entry.to_string_lossy().trim().replace(&repo_name, "");

            let target_path = Path::new(&target_path);
            if entry.is_dir() {
                rename(&entry, &target_path)?
            } else if entry.is_file() {
                rename(&entry, &target_path)?
            }
        }
        remove_dir(current_dir)?;
        Ok("下载成功!!!".dark_green().bold().to_string())
    }
}

fn check_name_is_valid(app_name: &String) -> anyhow::Result<()> {
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$")?;
    if !re.is_match(app_name) {
        bail!("Repo Name 格式无效,请使用字母、数字、下划线或连字符")
    }
    Ok(())
}

impl Buckets {
    pub fn display_all_buckets_extra(&self, is_global: bool) -> Result<(), anyhow::Error> {
        let (bucket_name, bucket_source, bucket_updated, bucket_manifest) =
            self.get_all_buckets(is_global)?;
        let combined_buckets = bucket_name
            .into_iter()
            .zip(bucket_source.into_iter())
            .zip(bucket_updated.into_iter())
            .zip(bucket_manifest.into_iter())
            .map(|(((name, source), updated), manifest)| vec![name, source, updated, manifest])
            .collect::<Vec<_>>();

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("BucketName")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
                Cell::new("SourceUrl")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
                Cell::new("UpdatedTime")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
                Cell::new("Manifests")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
            ])
            .add_rows(combined_buckets.as_slice());

        let column = table.column_mut(3).expect("Our table has three columns");
        column.set_cell_alignment(CellAlignment::Center);
        println!("{table}");
        Ok(())
    }
    pub fn display_all_buckets(&self, is_global: bool) -> Result<(), anyhow::Error> {
        let (bucket_name, bucket_source, bucket_updated, bucket_manifest) =
            self.get_all_buckets(is_global)?;

        let combined_buckets: Vec<(String, String, String, String)> = bucket_name
            .into_iter()
            .zip(bucket_source.into_iter())
            .zip(bucket_updated.into_iter())
            .zip(bucket_manifest.into_iter())
            .map(|(((name, source), updated), manifest)| (name, source, updated, manifest))
            .collect();
        let max_name_len = combined_buckets
            .iter()
            .map(|e| e.0.len())
            .max()
            .unwrap_or(0)
            + 10;
        let max_manifest_len = combined_buckets
            .iter()
            .map(|e| e.3.len())
            .max()
            .unwrap_or(0);
        let max_source_len = combined_buckets
            .iter()
            .map(|e| e.1.len())
            .max()
            .unwrap_or(0)
            + 4;
        let max_updated_len = combined_buckets
            .iter()
            .map(|e| e.2.len())
            .max()
            .unwrap_or(0)
            + 10;

        println!(
            "{:<max_name_len$}{}{:<max_source_len$}{}{:<max_updated_len$}{}{:<max_manifest_len$}",
            "BucketName".dark_cyan().bold(),
            " ".repeat(max_name_len - 10),
            "SourceUrl".dark_cyan().bold(),
            " ".repeat(max_source_len - 9),
            "UpdatedTime".dark_cyan().bold(),
            " ".repeat(max_updated_len - 11),
            "Manifests".dark_cyan().bold(),
            max_name_len = max_name_len,
            max_source_len = max_source_len,
            max_updated_len = max_updated_len,
            max_manifest_len = max_manifest_len
        );
        println!(
            "{:<max_name_len$}{}{:<max_source_len$}{}{:<max_updated_len$}{}{:<max_manifest_len$}",
            "__________".dark_cyan().bold(),
            " ".repeat(max_name_len - 10),
            "_________".dark_cyan().bold(),
            " ".repeat(max_source_len - 9),
            "___________".dark_cyan().bold(),
            " ".repeat(max_updated_len - 11),
            "_________".dark_cyan().bold(),
            max_name_len = max_name_len,
            max_source_len = max_source_len,
            max_updated_len = max_updated_len,
            max_manifest_len = max_manifest_len
        );
        for (name, source, updated, manifest) in combined_buckets.iter() {
            println!(
                "{:<max_name_len$}{:<max_source_len$}{:<max_updated_len$}{:<max_manifest_len$}",
                name,
                source,
                updated,
                manifest,
                max_name_len = max_name_len,
                max_source_len = max_source_len,
                max_updated_len = max_updated_len,
                max_manifest_len = max_manifest_len
            );
        }
        Ok(())
    }
    fn get_all_buckets(
        &self,
        is_global: bool,
    ) -> anyhow::Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
        let bucket_name = if is_global {
            get_global_all_buckets_name()?
        } else {
            get_buckets_name()?
        };
        let bucket_source_url = self.get_bucket_source_url(is_global)?;
        if bucket_name.is_empty() || bucket_source_url.is_empty() {
            bail!("buckets dir is not exist")
        }
        let bucket_source = if is_global {
            get_global_all_buckets_dir()?
        } else {
            get_buckets_path()?
        };

        let bucket_updated = Self::get_updated_time(&bucket_source)?;
        let bucket_manifest = Self::get_manifest_version(&bucket_source)?;
        Ok((
            bucket_name,
            bucket_source_url,
            bucket_updated,
            bucket_manifest,
        ))
    }

    pub fn get_updated_time(bucket_source: &Vec<String>) -> anyhow::Result<Vec<String>> {
        let mut bucket_updated: Vec<String> = Vec::new();
        for source in bucket_source {
            let path = source.to_string() + "\\bucket";
            if !Path::new(&path).exists() {
                bail!("bucket path {} does not exist", path);
            }
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
        Ok(bucket_updated)
    }

    pub fn get_manifest_version(path: &Vec<String>) -> anyhow::Result<Vec<String>> {
        let mut bucket_manifest: Vec<String> = Vec::new();
        // 获取目录的子文件个数
        for source in path {
            let source = source.to_string() + "\\bucket";
            if !Path::new(&source).exists() {
                bail!("bucket dir {} does not exist", source);
            }
            let count = read_dir(source)?.count(); // 这里得到的是一个`u64`
            bucket_manifest.push(count.to_string());
        }

        Ok(bucket_manifest)
    }

    pub fn get_bucket_source_url(&self, is_global: bool) -> anyhow::Result<Vec<String>> {
        let bucket_path = if is_global {
            get_global_all_buckets_dir()?
        } else {
            get_buckets_path()?
        };
        let result = bucket_path.iter().try_for_each(|path| {
            if !Path::new(path).exists() || !Path::new(path).is_dir() {
                bail!("bucket dir not found")
            } else {
                Ok(())
            }
        });
        if result.is_err() {
            bail!(result.err().unwrap())
        }
        let buckets_path = bucket_path
            .iter()
            .map(|path| get_git_repo_remote_url(path).unwrap())
            .collect::<Vec<_>>();

        Ok(buckets_path)
    }
    pub fn get_bucket_source_url_and_path(
        &self,
        is_global: bool,
    ) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let bucket_paths = if is_global {
            get_global_all_buckets_dir()?
        } else {
            get_buckets_path()?
        };
        let result = bucket_paths.iter().try_for_each(|path| {
            if !Path::new(path).exists() || !Path::new(path).is_dir() {
                bail!("bucket dir not found")
            } else {
                Ok(())
            }
        });
        if result.is_err() {
            bail!(result.err().unwrap())
        }
        let buckets_path = bucket_paths
            .iter()
            .map(|path| get_git_repo_remote_url(path).unwrap())
            .collect::<Vec<_>>();

        Ok((buckets_path, bucket_paths))
    }
}

pub fn get_hp_bucket_repo_path(bucket_name: &str) -> anyhow::Result<Option<String>> {
    let bucket = Buckets::new()?;
    let (urls, bucket_paths) = bucket.get_bucket_source_url_and_path(false)?;
    if bucket_name.to_lowercase() == "hp" {
        let result = urls
            .into_iter()
            .zip(bucket_paths.into_iter())
            .find(|(url, _)| url.contains("hyperscoop_bucket"))
            .ok_or_else(|| anyhow!("未找到 hyperscoop_bucket 仓库"))
            as anyhow::Result<(String, String)>;

        if result.is_err() {
            bail!(result.err().unwrap())
        }
        let path = result?.1;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

impl Buckets {
    pub fn get_bucket_self(&self) -> Result<(Vec<String>, Vec<String>), anyhow::Error> {
        let bucket = Buckets::new()?;
        Ok((bucket.buckets_path, bucket.buckets_name))
    }
    pub fn get_global_bucket_self(&self) -> Result<(Vec<String>, Vec<String>), anyhow::Error> {
        let bucket = Buckets::new()?;
        Ok((bucket.global_buckets_paths, bucket.global_buckets_names))
    }
    pub fn new() -> anyhow::Result<Buckets> {
        let bucket_path = get_buckets_root_dir_path();
        // 遍历 bucket_path 下的所有文件夹，并将文件夹名加入 buckets_path
        let buckets_path: Vec<String> = read_dir(&bucket_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path().to_str().unwrap().to_string())
            .collect();
        let buckets_name: Vec<String> = buckets_path
            .iter()
            .map(|e| e.split("\\").last().unwrap().to_string())
            .collect();
        let global_buckets_paths = get_buckets_root_dir_path_global();
        if Path::new(&global_buckets_paths).exists() {
            let global_buckets_paths: Vec<String> = read_dir(&global_buckets_paths)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path().to_str().unwrap().to_string())
                .collect();
            let global_buckets_names = global_buckets_paths
                .iter()
                .map(|e| e.split("\\").last().unwrap().to_string())
                .collect();
            Ok(Buckets {
                buckets_path,
                buckets_name,
                global_buckets_paths,
                global_buckets_names,
            })
        } else {
            Ok(Buckets {
                buckets_path,
                buckets_name,
                global_buckets_paths: vec![],
                global_buckets_names: vec![],
            })
        }
    }

    pub fn get_known_bucket_path(&self) -> String {
        let home_dir = std::env::var("USERPROFILE").unwrap();
        let config_dir = home_dir + "\\.config\\scoop";
        if !Path::new(&config_dir).exists() {
            create_dir_all(&config_dir).unwrap();
        }
        let known = format!("{}\\known_bucket.json", config_dir);
        let known_bucket = Path::new(&known);
        if !known_bucket.exists() {
            let mut file = File::create(&known_bucket).unwrap(); // 截断写入
            let content = r#"{
    "main": "https://github.com/ScoopInstaller/Main",
    "extras": "https://github.com/ScoopInstaller/Extras",
    "versions": "https://github.com/ScoopInstaller/Versions",
    "nirsoft": "https://github.com/ScoopInstaller/Nirsoft",
    "sysinternals": "https://github.com/niheaven/scoop-sysinternals",
    "php": "https://github.com/ScoopInstaller/PHP",
    "nerd-fonts": "https://github.com/matthewjberger/scoop-nerd-fonts",
    "nonportable": "https://github.com/ScoopInstaller/Nonportable",
    "java": "https://github.com/ScoopInstaller/Java",
    "games": "https://github.com/Calinou/scoop-games"
}
            "#;
            file.write_all(content.as_bytes()).unwrap();
            file.flush().unwrap();
        }
        known
    }
    pub fn get_bucket_known(
        &self,
        is_global: bool,
    ) -> Result<(Vec<String>, Vec<String>), anyhow::Error> {
        log::warn!("global {is_global}");
        let apps_dir = if is_global {
            get_apps_path_global()
        } else {
            get_apps_path()
        };
        if !Path::new(&apps_dir).exists() {
            bail!("global {apps_dir} is not  exist");
        }
        let known_bucket_path = self.get_known_bucket_path();
        let file_buffer = File::open(&known_bucket_path).expect("Failed to open known_bucket_path");
        let reader_buffer = BufReader::new(file_buffer);
        let content: serde_json::Value = serde_json::from_reader(reader_buffer)?;
        let mut known_name: Vec<String> = Vec::new();
        let mut known_source: Vec<String> = Vec::new();
        let re = Regex::new(r#""(https?://\S+)""#)?;
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
        Ok((known_name, known_source))
    }

    pub fn display_known_buckets(&self, is_global: bool) -> Result<(), anyhow::Error> {
        let (known_name, known_source) = self.get_bucket_known(is_global)?;
        let max_name_len = known_name.iter().map(|e| e.len()).max().unwrap_or(0) + 10;
        let interval_len = max_name_len - 10 + 1;
        println!(
            "{}{}{}",
            "BucketName".dark_green().bold(),
            " ".repeat(interval_len),
            "SourceUrl  ".dark_green().bold()
        );
        for (name, source) in known_name.iter().zip(known_source.iter()) {
            println!(
                "{:<max_name_len$} {:<max_name_len$}",
                name.to_string(),
                source.to_string(),
                max_name_len = max_name_len
            );
        }
        Ok(())
    }
}

mod tests_buckets {

    #[test]
    fn test_get_hp_bucket() {
        use crate::buckets::get_hp_bucket_repo_path;
        let path = get_hp_bucket_repo_path("hp").unwrap();
        if path.is_some() {
            println!("hp bucket path: {}", path.unwrap());
        }
    }
}
