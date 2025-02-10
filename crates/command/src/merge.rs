use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::utils::detect_encoding::transform_to_search_manifest_object;
use anyhow::{anyhow, bail};
use crossterm::style::Stylize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::error;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use crate::update::update_scoop_bar;

#[derive(Debug, Eq, PartialEq, Hash, Clone)] // 从引用clone出新的完整对象而不是引用
struct Merge {
    pub app_name: String,
    pub app_version: String,
}

impl Merge {
    pub fn new(app_name: &String, app_version: &String) -> Self {
        Merge {
            app_name: app_name.clone(),
            app_version: app_version.clone(),
        }
    }
}
impl ToString for Merge {
    fn to_string(&self) -> String {
        format!(
            "{}   :  {}",
            self.app_name.clone().dark_blue().bold(),
            self.app_version.clone().dark_blue().bold()
        )
    }
}

// 合并所有冗余的manifest
pub fn merge_all_buckets() -> Result<(), anyhow::Error> {
    //  1. 读取所有bucket的manifest文件
    println!("{ }", "正在合并所有冗余的manifest文件".dark_green().bold());
    let paths = get_buckets_path()?;
    let mut paths = paths
        .iter()
        .map(|item| item.to_string() + "\\bucket")
        .collect::<Vec<String>>();
    paths.reverse();
    //  初始化容器
    let mut all_bucket_set: HashMap<String, Merge> = HashMap::new();
    // 记录所有旧版本的容器
    for path in &paths {
        let path_dir = Path::new(path);
        if path_dir.is_dir() {
            load_bucket_info(path_dir, &mut all_bucket_set)?;
        }
    }

    let latest_buckets: Vec<Merge> = all_bucket_set.values().cloned().collect();
    let mut latest_buckets_map: HashMap<String, Merge> = HashMap::new();
    let mut all_manifest = Vec::new();
    for path in &paths {
        let path_dir = Path::new(path);
        if path_dir.is_dir() {
            let manifest = remove_old_manifest(path_dir, &latest_buckets, &mut latest_buckets_map)
                .expect("删除旧版本manifest失败");

            if !manifest.is_empty() {
                all_manifest.push(manifest);
            }
        }
    }
    merge_same_latest_version(all_manifest);

    println!("{ }", "合并完成".dark_green().bold());
    Ok(())
}

fn load_bucket_info(
    path_dir: &Path,
    map: &mut HashMap<String, Merge>,
) -> Result<(), anyhow::Error> {
    if !path_dir.is_dir() {
        return Err(anyhow!("路径不是目录"));
    }
    let path = exclude_special_dir(path_dir);
    if let Err(_e) = path {
        return Ok(());
    }
    let path = path?;
    println!(
        "加载bucket：{}",
        &path
            .to_str()
            .expect("Invalid path")
            .to_string()
            .dark_blue()
            .bold()
    );
    for entry in path.read_dir()? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        let path = entry.path();
        if path.is_dir() {
            // println!("{ } {} ", "跳过目录".dark_green().bold(),
            //          file_name.to_str().expect("Invalid file name").to_string().dark_blue().bold());
            continue;
        } else if path.is_file() && exclude_not_json_file(file_name) {
            // println!("{ } {}", "跳过非json文件".dark_green().bold(),
            //          file_name.to_str().unwrap().to_string().dark_blue().bold());
            continue;
        } else if path.is_file()
            && path.extension().is_some()
            && path.to_string_lossy().to_string().ends_with(".json")
        {
            // 对于 path使用ends_with 只能匹配路径的最后一个元素,不能匹配扩展名
            // println!("{ } {}", "正在读取文件".dark_green().bold(), file_name.to_str().unwrap().to_string().dark_blue().bold());

            let result = extract_info_from_manifest(&path)?;
            find_latest_version(result, map).expect("执行合并失败");
        } else {
            print!("{}", path.to_str().unwrap().to_string().dark_blue().bold());
            error!("文件类型不支持");
            return Err(anyhow!("该文件不存在"));
        }
    }
    Ok(())
}
fn exclude_special_dir(path_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let exclude_dirs = [
        "main",
        "extras",
        "versions",
        "nirsoft",
        "sysinternals",
        "php",
        "nerd-fonts",
        "nonportable",
        "java",
        "games",
        "dorado",
        "DoveBoyApps",
        "echo",
        "lemon",
        "Python",
        "samiya",
    ];
    for exclude_dir in exclude_dirs {
        if path_dir.to_str().unwrap().contains(exclude_dir) {
            return Err(anyhow!("排除目录"));
        }
    }
    Ok(path_dir.to_path_buf())
}
fn find_latest_version(
    merge: Merge,
    map_container: &mut HashMap<String, Merge>,
) -> Result<(), anyhow::Error> {
    // 存入集合
    //  如果变量定义在循环中会导致变量遮蔽
    //如果merge任意字段为空，则跳过
    if merge.app_version.is_empty() || merge.app_version.contains("null") {
        println!(
            "{}  :  {}",
            merge.app_name.clone().dark_blue().bold(),
            merge.app_version.clone().dark_blue().bold()
        );
        return Ok(());
    }
    // 先找到最高版本, 第二部删除旧版本
    if !map_container.contains_key(&merge.app_name) {
        let result = map_container.insert(merge.app_name.to_string(), merge);
        if let Some(result) = result {
            println!("{}", result.to_string().dark_blue().bold());
        }
        //  insert插入的键不存在时，返回None,所有不能进行错误处理  , 更新旧值返回旧值
    } else {
        //  print!("第一个冗余manifest");
        let old_bucket = map_container
            .get(&merge.app_name)
            .ok_or(anyhow!("No app version"))
            .expect("不存在这个merge ");
        let old_app_version = old_bucket.app_version.to_string();
        let new_app_versio = merge.app_version.to_string();
        //  insert 会自动覆盖旧值
        if new_app_versio > old_app_version {
            map_container.insert(new_app_versio, merge);
        }
    };
    Ok(())
}

fn remove_old_manifest(
    bucket_dir: &Path,
    latest_buckets: &Vec<Merge>,
    latest_buckets_map: &mut HashMap<String, Merge>,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let bucket_dir = exclude_special_dir(bucket_dir);
    if let Err(_e) = bucket_dir {
        return Ok(vec![]);
    }
    // 将 latest_buckets 转换为HashMap

    for item in latest_buckets {
        latest_buckets_map.insert(item.app_name.to_string(), item.clone());
    }

    let bucket_dir = bucket_dir?;
    let mut same_latest_version_manifests = vec![];
    for entry in bucket_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if !path.exists() {
            continue;
        }
        if path.is_file() && path.to_string_lossy().to_string().ends_with(".json") {
            let app_name = path.file_stem().unwrap().to_string_lossy().to_string();
            let app_name = app_name.split("/").last().expect("Invalid path");
            if latest_buckets_map.contains_key(app_name) {
                let json_str = transform_to_search_manifest_object(&path).expect("文件解析错误");
                let app_version = json_str.get_version().unwrap();

                if app_version.to_string() < latest_buckets_map.get(app_name).unwrap().app_version {
                    //  println!("删除的文件{} 版本{}", path.display(), app_version);
                    remove_file(&path).expect("删除文件失败");
                } else {
                    //多个相等的manifest最高版本只保留一个
                    same_latest_version_manifests.push(path);
                }
            }
        }
    }
    Ok(same_latest_version_manifests)
}

fn merge_same_latest_version(same_latest_version_manifests: Vec<Vec<PathBuf>>) {
    let latest_manifest = &same_latest_version_manifests.clone();
    let mut group_manifests = HashMap::new();
    let mut all_manifest_count = 0;
    for manifests in latest_manifest {
        for manifest in manifests.iter() {
            all_manifest_count += 1;
            let name = manifest.file_stem().unwrap().to_string_lossy().to_string();
            let app_name = name.split("/").last().unwrap().to_string();
            if !group_manifests.contains_key(&app_name) {
                group_manifests.insert(app_name, 1);
            } else {
                let count = *group_manifests.get(&app_name).unwrap() + 1;
                group_manifests.insert(app_name, count);
            }
        }
    }
    // 初始化进度条
    let total_manifest_count = all_manifest_count;
    let styles = [
        ("Rough bar:", "█  ", "red"),
        ("Fine bar: ", "█▉▊▋▌▍▎▏  ", "yellow"),
        ("Vertical: ", "█▇▆▅▄▃▂▁  ", "green"),
        ("Fade in:  ", "█▓▒░  ", "blue"),
        ("Blocky:   ", "█▛▌▖  ", "magenta"),
    ];
    // let group_manifests_ref = Arc::new(Mutex::new(group_manifests));
    let m = MultiProgress::new();
    for manifests in latest_manifest {
        for manifest in manifests.iter() {
            let name = manifest.file_stem().unwrap().to_string_lossy().to_string();
            let app_name = name.split("/").last().unwrap().to_string();
            // 异步读取 count
            let count = group_manifests
                .get(&app_name)
                .ok_or(anyhow!("应用不存在"))
                .unwrap()
                .clone();

            if count > 1 && manifest.exists() {
                remove_file(manifest).expect("删除文件失败");
                group_manifests.insert(app_name.clone(), count - 1);
            }
        }
    }

    //-----

    let handles: Vec<_> = styles
        .iter()
        .map(|s| {
            let pb = m.add(ProgressBar::new(total_manifest_count as u64));
            pb.set_style(
                ProgressStyle::with_template(&format!("{{prefix:.bold}}▕{{bar:.{}}}▏{{msg}}", s.2))
                    .unwrap()
                    .progress_chars(s.1),
            );
            pb.set_prefix(s.0);
            let wait = Duration::from_millis(thread_rng().gen_range(10..20));
            thread::spawn(move || {
                for i in 0..(total_manifest_count / 50) {
                    thread::sleep(wait);
                    pb.inc(50);
                    pb.set_message(format!("{:3}%", 5000 * i / total_manifest_count));
                }
                pb.finish_with_message("处理完成");
            })
        })
        .collect();

    for h in handles {
        let _ = h.join();
    }

    //   println!("检验manifest数量 \n{:? }", group_manifests);
}
#[allow(unused)]
fn finebars(file_finish_count: u64, total_file_count: u64) {
    let styles = [
        ("Rough bar:", "█  ", "red"),
        ("Fine bar: ", "█▉▊▋▌▍▎▏  ", "yellow"),
        ("Vertical: ", "█▇▆▅▄▃▂▁  ", "green"),
        ("Fade in:  ", "█▓▒░  ", "blue"),
        ("Blocky:   ", "█▛▌▖  ", "magenta"),
    ];

    let m = MultiProgress::new();

    let handles: Vec<_> = styles
        .iter()
        .map(|s| {
            let pb = m.add(ProgressBar::new(total_file_count as u64));
            pb.set_style(
                ProgressStyle::with_template(&format!("{{prefix:.bold}}▕{{bar:.{}}}▏{{msg}}", s.2))
                    .unwrap()
                    .progress_chars(s.1),
            );
            pb.set_prefix(s.0);
            let wait = Duration::from_millis(thread_rng().gen_range(10..20));
            thread::spawn(move || {
                thread::sleep(wait);
                let move_rate = 1000 / total_file_count;
                pb.inc(move_rate * 100);
                pb.set_message(format!("{:3}%", file_finish_count / total_file_count));
                pb.finish_with_message("100%");
            })
        })
        .collect();

    for h in handles {
        let _ = h.join();
    }
}

fn extract_info_from_manifest(path: &PathBuf) -> Result<Merge, anyhow::Error> {
    // println!("正在读取文件：{}", path.to_str().unwrap().to_string().dark_blue().bold());

    let manifest_json = transform_to_search_manifest_object(path).expect("文件解析错误");

    let app_version = manifest_json.version.unwrap_or_default();
    // file_stem 去掉文件的扩展名
    if app_version.is_empty() || app_version.contains("null") {
        println!("删除无效文件{}", path.display());
        remove_file(path).expect("删除文件失败");
    }
    let app_name = path.file_stem().unwrap().to_string_lossy().to_string();
    let app_name = app_name
        .split("/")
        .last()
        .expect("Invalid path")
        .trim()
        .to_string();
    let merge = Merge::new(&app_name, &app_version);
    Ok(merge)
}
#[allow(unused)]
fn display_repeat_app(merge: &Merge) {
    let app_name = merge.app_name.clone();
    let mut app_set = HashSet::new();
    if !app_set.insert(&app_name) {
        println!("{} 重复", app_name.clone().dark_blue().bold());
    }
}

fn exclude_not_json_file(file_name: String) -> bool {
    // 排除非json文件 , 匹配 .开头和_开头的文件
    if file_name.starts_with(".") || file_name.starts_with("_") {
        return true;
    } else if !file_name.ends_with(".json") {
        return true;
    }
    false
}


pub fn rm_err_manifest() -> Result<(), anyhow::Error> {
  use  crate::utils::progrees_bar::{
    indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle},
    style, Message, ProgressOptions,
  } ;
  const FINISH_MESSAGE: &'static str = "✅";
  let progress_style = style(Some(ProgressOptions::Hide), Some(Message::suffix()));
  let bucket_paths= get_buckets_path()?;
  let buckets_name = get_buckets_name()?;

  let mp = MultiProgress::new();
  let  longest_bucket_name =
    buckets_name.iter().map( |item | item.len()).max().unwrap_or(0) ;
  let outdated_buckets = buckets_name
    .into_iter()
    .map(|bucket| {
      let pb = mp.add(
        ProgressBar::new(1)
          .with_style(progress_style.clone())
          .with_message("Checking updates")
          .with_prefix(format!("🪣 {:<longest_bucket_name$}", bucket))
          .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into())),
      );
      pb.set_position(0);
      (bucket, pb)
    }).collect::<Vec<_>>();

  Ok(())
}
