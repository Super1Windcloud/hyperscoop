use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::manifest::search_manifest::SearchManifest;
use crate::utils::request::get_git_repo_remote_url;
use crate::utils::utility::{LARGE_COMMUNITY_BUCKET, remove_bom_and_control_chars_from_utf8_file};
use anyhow::{Context, anyhow, bail};
use crossterm::style::Stylize;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressFinish, ProgressStyle};
use rand::{Rng, thread_rng};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::{read_to_string, remove_file};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Merge {
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
impl Display for Merge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!(
            "{}   :  {}",
            self.app_name.clone().dark_blue().bold(),
            self.app_version.clone().dark_blue().bold()
        );
        write!(f, "{}", str)
    }
}

pub fn merge_all_buckets() -> Result<(), anyhow::Error> {
    //  1. 读取所有bucket的manifest文件
    println!("{ }", "正在合并所有冗余的manifest文件".dark_green().bold());
    let paths = get_buckets_path()?;
    let paths = paths
        .iter()
        .map(|item| item.to_string() + "\\bucket")
        .collect::<Vec<String>>();
    //  初始化容器
    let all_bucket_set = Mutex::new(HashMap::<String, Merge>::new());
    // 记录所有旧版本的容器
    paths.par_iter().for_each(|path| {
        let path_dir = Path::new(path);
        if path_dir.is_dir() {
            load_bucket_info(path_dir, &all_bucket_set).expect("加载bucket失败");
        }
    });
    let latest_buckets: Vec<Merge> = all_bucket_set.lock().unwrap().values().cloned().collect();
    let all_manifest = Mutex::new(Vec::new());
    paths.par_iter().for_each(|path| {
        let path_dir = Path::new(path);
        if path_dir.is_dir() {
            let manifest =
                remove_old_manifest(path_dir, &latest_buckets).expect("删除旧版本manifest失败");

            if !manifest.is_empty() {
                all_manifest.lock().unwrap().push(manifest);
            }
        }
    });

    let all_manifest = all_manifest
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    merge_same_latest_version(all_manifest)?;

    println!("{ }", "合并完成".dark_green().bold());
    Ok(())
}

fn load_bucket_info(
    path_dir: &Path,
    map: &Mutex<HashMap<String, Merge>>,
) -> Result<(), anyhow::Error> {
    let path = include_special_dir(path_dir);
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
    let entries = path
        .read_dir()
        .context("read bucket dir error at line 109")?
        .par_bridge()
        .collect::<Result<Vec<_>, _>>()?;
    let result = entries
        .par_iter()
        .map(|entry| {
            let path = entry.path();
            let file_type = entry.file_type().ok()?;
            return if file_type.is_file()
                && path.extension().and_then(|ext| ext.to_str()) == Some("json")
            {
                // 对于 path使用ends_with 只能匹配路径的最后一个元素,不能匹配扩展名
                let result = extract_info_from_manifest(&path);
                if let Err(e) = result {
                    eprintln!("{}", e.to_string().dark_blue().bold());
                    return None;
                }
                let result = result.unwrap();
                Some(result)
            } else {
                None
            };
        })
        .collect::<Vec<_>>();

    result.into_par_iter().for_each(|merge| {
        if merge.is_none() {
            return;
        }
        let merge = merge.unwrap();
        let mut map = map.lock().unwrap();
        find_latest_version(merge, &mut map).expect("执行合并失败");
    });
    Ok(())
}
fn include_special_dir(path_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let repo_path = path_dir.to_str().unwrap().strip_suffix("bucket");
    if repo_path.is_none() {
        return Err(anyhow!("路径不是目录"));
    }
    let repo_path = PathBuf::from(repo_path.unwrap());
    if !repo_path.exists() {
        return Err(anyhow!("路径不存在"));
    }
    let remote_url = get_git_repo_remote_url(repo_path)?;
    if !LARGE_COMMUNITY_BUCKET.contains(&remote_url.as_str()) {
        return Err(anyhow!("排除目录"));
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
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let latest_buckets_map: Arc<Mutex<HashMap<String, Merge>>> =
        Arc::new(Mutex::new(HashMap::<String, Merge>::new()));
    let bucket_dir = include_special_dir(bucket_dir);
    if let Err(_) = bucket_dir {
        return Ok(vec![]);
    }
    // 将 latest_buckets 转换为HashMap

    latest_buckets.par_iter().for_each(|item| {
        latest_buckets_map
            .lock()
            .unwrap()
            .insert(item.app_name.to_string(), item.clone());
    });

    let bucket_dir = bucket_dir?;

    // 读取目录条目并收集结果（提前处理错误）
    let entries: Vec<_> = bucket_dir
        .read_dir()
        .context("read bucket dir error at line 220")?
        .par_bridge()
        .collect::<Result<Vec<_>, _>>()?;

    let same_latest_version_manifests = entries
        .into_par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let file_type = entry.file_type().ok()?;
            if !file_type.is_file() {
                return None;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let app_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .expect("Invalid file stem");
                let app_name = app_name.split("/").last().expect("Invalid path");
                if latest_buckets_map.lock().unwrap().contains_key(app_name) {
                    let content = read_to_string(&path).unwrap_or_default();
                    if content.is_empty() {
                        remove_file(&path).expect("删除文件失败");
                        return None;
                    }
                    let json_str: SearchManifest =
                        serde_json::from_str(&content).unwrap_or_default();

                    let app_version = json_str.get_version().unwrap_or_default();
                    if app_version.is_empty() {
                        remove_file(&path).expect("删除文件失败");
                        return None;
                    }
                    return if app_version.to_string()
                        < latest_buckets_map
                            .lock()
                            .unwrap()
                            .get(app_name)
                            .unwrap()
                            .app_version
                    {
                        //  println!("删除的文件{} 版本{}", path.display(), app_version);
                        remove_file(&path).expect("删除文件失败");
                        None
                    } else {
                        //多个相等的manifest最高版本只保留一个
                        path.into()
                    };
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();
    Ok(same_latest_version_manifests)
}

fn merge_same_latest_version(
    mut same_latest_version_manifests: Vec<PathBuf>,
) -> Result<(), anyhow::Error> {
    use dashmap::DashMap;

    let group_manifests = DashMap::new();
    let total_manifest_count = same_latest_version_manifests.len();

    let (scoop_master, other): (Vec<_>, Vec<_>) = same_latest_version_manifests
        .into_par_iter()
        .partition(|manifest| {
            let name = manifest.to_str().unwrap();
            if name.contains("ScoopMaster") {
                true
            } else {
                false
            }
        });
    // ? cloned 获取引用指向的值,并将值复制到新的变量中 ,转换&  Vec<T> 到 Vec<T>
    same_latest_version_manifests = scoop_master
        .into_iter()
        .chain(other.into_iter())
        .collect::<_>();
    same_latest_version_manifests
        .par_iter()
        .for_each(|manifest| {
            let name = manifest.file_stem().unwrap().to_string_lossy().to_string();
            let app_name = name.split("/").last().unwrap().to_string();
            *group_manifests.entry(app_name).or_insert(0) += 1;
        });
    let pb = ProgressBar::new(total_manifest_count as u64)
        .with_style(
            ProgressStyle::default_bar()
                .template(
                    "{prefix}  {spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}",
                )?
                .progress_chars("#>-"),
        )
        .with_prefix(format!("🐎 {:<10}", "ProgressBar"))
        .with_message("Remove Redundant Manifests")
        .with_finish(ProgressFinish::WithMessage("Done 🎉".into()));
    pb.set_draw_target(ProgressDrawTarget::stdout());

    // 并行处理文件
    let _ = same_latest_version_manifests
        .into_par_iter()
        .map(|manifest| {
            let name = manifest.file_stem().unwrap().to_string_lossy().to_string();
            let app_name = name.split("/").last().unwrap().to_string();
            let count = group_manifests
                .get(&app_name)
                .ok_or(anyhow!("应用不存在"))
                .unwrap()
                .clone();

            if count > 1 && manifest.exists() {
                // write_into_log_file(&manifest);
                remove_file(&manifest).expect("删除文件失败");
                group_manifests.insert(app_name.clone(), count - 1);
            }
            pb.inc(1);
        })
        .collect::<Vec<()>>();
    Ok(())
}

fn extract_info_from_manifest(path: &PathBuf) -> Result<Merge, anyhow::Error> {
    let content = read_to_string(path).unwrap_or_default();
    if content.is_empty() {
        return Err(anyhow!("文件为空"));
    }
    let manifest_json: SearchManifest = serde_json::from_str(&content).unwrap_or_default();

    let app_version = manifest_json.version.unwrap_or_default();
    // file_stem 去掉文件的扩展名
    if app_version.is_empty() {
        println!("删除无效文件{}", path.display());
        remove_file(path).expect("删除文件失败");
    }
    let app_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Invalid file stem");
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
#[allow(dead_code)]
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
    use crate::utils::progrees_bar::indicatif::{MultiProgress, ProgressBar, ProgressFinish};
    const FINISH_MESSAGE: &str = "✅";
    let bucket_paths = get_buckets_path()?;
    let buckets_name = get_buckets_name()?;

    let bucket_manifests_count = bucket_paths
        .par_iter()
        .map(|path| {
            let path = Path::new(path);
            let paths = path.join("bucket");
            let entry = paths.read_dir().unwrap();
            let bucket_name = path.file_name().unwrap().to_string_lossy().to_string();
            (bucket_name, entry.count())
        })
        .collect::<Vec<_>>();
    let mp = MultiProgress::new();
    let longest_bucket_name = buckets_name
        .iter()
        .map(|item| item.len())
        .max()
        .unwrap_or(0);

    let outdated_buckets = buckets_name
    .into_iter()
    .map(|bucket| {
      let count = bucket_manifests_count.iter().find(|item| item.0 == bucket).unwrap().1;
      let pb = mp.add(
        ProgressBar::new(count as u64).with_style(
          ProgressStyle::default_bar()
            .template("{prefix}  {spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}").unwrap()
            .progress_chars("#>-")
        )
          .with_prefix(format!("🐼 {:<longest_bucket_name$}", bucket))
          .with_message("Remove Error Manifests")
          .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into())),
      );

      pb.set_position(0);
      pb.set_draw_target(ProgressDrawTarget::stdout());

      (bucket, pb)
    }).collect::<Vec<_>>();

    let validate = outdated_buckets
        .par_iter()
        .map(|(bucket, pb)| {
            let bucket_paths = bucket_paths
                .par_iter()
                .map(|path| {
                    let path = Path::new(path);
                    let paths = path.join("bucket");
                    paths.to_str().unwrap().to_string()
                })
                .collect::<Vec<_>>();
            let bucket_path = bucket_paths
                .iter()
                .find(|item| item.ends_with(&(bucket.clone() + "\\bucket")))
                .unwrap_or(bucket);
            let result = rm_err_manifest_unit(bucket_path, pb, FINISH_MESSAGE.parse().unwrap());
            if let Err(e) = result {
                pb.finish_with_message(format!("❌ {}", e.to_string()));
            }
            bucket_path.into()
        })
        .collect::<Vec<String>>();
    validate.par_iter().for_each(|path| {
        if !Path::new(path).exists() {
            eprintln!("{} 不存在", path.clone().dark_red().bold());
        }
    });
    Ok(())
}

fn rm_err_manifest_unit(
    bucket_path: &String,
    pb: &ProgressBar,
    finish_message: String,
) -> anyhow::Result<()> {
    let git_repo = bucket_path.strip_suffix("bucket").unwrap();
    let repo_path = Path::new(git_repo);
    if !repo_path.exists() {
        bail!(
            "{} 不存在",
            repo_path.to_str().unwrap().to_string().dark_red().bold()
        );
    }
    let bucket_path = Path::new(bucket_path);
    let manifests = bucket_path
        .read_dir()
        .context(format!(
            "read bucket dir error at line 479 {}",
            bucket_path.display()
        ))?
        .par_bridge()
        .filter_map(|path| Some(path.ok()))
        .collect::<Vec<_>>();
    let git_url = get_git_repo_remote_url(repo_path).unwrap_or_default();
    if git_url.is_empty() {
        bail!(
            "{} 不是git仓库",
            bucket_path.to_str().unwrap().to_string().dark_red().bold()
        );
    }
    manifests.par_iter().for_each(|manifest_path| {
        pb.inc(1);
        if manifest_path.is_none() {
            return;
        }
        let manifest_path = manifest_path.as_ref().unwrap().path();
        if manifest_path.is_file() && manifest_path.clone().to_str().unwrap().ends_with(".json") {
            let content = read_to_string(&manifest_path).unwrap_or_default();
            if content.is_empty() {
                remove_file(&manifest_path).unwrap_or_else(|_| {
                    eprintln!(
                        "{} 删除失败",
                        manifest_path
                            .to_str()
                            .unwrap()
                            .to_string()
                            .dark_red()
                            .bold()
                    );
                });
                // crate::utils::utility::write_into_log_file(&manifest_path);
                return;
            }

            if LARGE_COMMUNITY_BUCKET.contains(&git_url.as_str()) {
                let content = remove_bom_and_control_chars_from_utf8_file(&manifest_path);
                if content.is_err() {
                    remove_file(&manifest_path).unwrap_or_else(|_| {
                        eprintln!(
                            "{} 删除失败",
                            manifest_path
                                .to_str()
                                .unwrap()
                                .to_string()
                                .dark_red()
                                .bold()
                        );
                    });
                    // crate::utils::utility::write_into_log_file(&manifest_path);
                    return;
                }
                let content = serde_json::from_str::<serde_json::Value>(&content.unwrap())
                    .unwrap_or_default();
                if content.is_null() {
                    remove_file(&manifest_path).unwrap_or_else(|_| {
                        eprintln!(
                            "{} 删除失败",
                            manifest_path
                                .to_str()
                                .unwrap()
                                .to_string()
                                .dark_red()
                                .bold()
                        );
                    });
                    // crate::utils::utility::write_into_log_file(&manifest_path);
                    return;
                }
                return;
            }
            let content = read_to_string(&manifest_path);
            if content.is_err() {
                remove_file(&manifest_path).unwrap_or_else(|_| {
                    eprintln!(
                        "{} 删除失败",
                        manifest_path
                            .to_str()
                            .unwrap()
                            .to_string()
                            .dark_red()
                            .bold()
                    );
                });
                // crate::utils::utility::write_into_log_file(&manifest_path);
                return;
            }
            let content =
                serde_json::from_str::<serde_json::Value>(&content.unwrap()).unwrap_or_default();
            if content.is_null() {
                remove_file(&manifest_path).unwrap_or_else(|_| {
                    eprintln!(
                        "{} 删除失败",
                        manifest_path
                            .to_str()
                            .unwrap()
                            .to_string()
                            .dark_red()
                            .bold()
                    );
                });
                // crate::utils::utility::write_into_log_file(&manifest_path);
                return;
            }
        }
    });
    pb.finish_with_message(finish_message);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_transfrom_result() {
        let file = r"A:\Scoop\buckets\echo\bucket\hdtune.json";
        let content = std::fs::read_to_string(file).unwrap();
        let result = serde_json::from_str::<serde_json::Value>(&content).unwrap();
        println!("{:?}", result);
    }
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
            let pb = m.add(ProgressBar::new(total_file_count));
            pb.set_style(
                ProgressStyle::with_template(&format!("{{prefix:.bold}}▕{{bar:.{}}}▏{{msg}}", s.2))
                    .unwrap()
                    .progress_chars(s.1),
            );
            pb.set_draw_target(ProgressDrawTarget::stdout());

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
