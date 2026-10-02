use crate::buckets::Buckets;
use crate::i18n::tr;
use crate::init_env::{
    get_app_dir_install_json, get_app_dir_install_json_global, get_apps_path, get_apps_path_global,
    is_app_held_by_path,
};
use crate::install::{InstallOptions, install_app, install_from_specific_bucket};
use crate::list::get_install_json_bucket;
use crate::tr_fmt;
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use std::collections::HashSet;
use std::fs::read_dir;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleBucket {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleApp {
    pub name: String,
    pub bucket: Option<String>,
    pub global: bool,
    pub hold: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hpfile {
    pub buckets: Vec<BundleBucket>,
    pub apps: Vec<BundleApp>,
}

impl Hpfile {
    /// Parse an Hpfile content string into an `Hpfile` struct
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut buckets = Vec::new();
        let mut apps = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "bucket" => {
                    if parts.len() < 2 {
                        bail!("Line {line_num}: 'bucket' requires a bucket name");
                    }
                    let name = parts[1].to_string();
                    let url = if parts.len() >= 3 {
                        Some(parts[2].to_string())
                    } else {
                        None
                    };
                    buckets.push(BundleBucket { name, url });
                }
                "app" => {
                    if parts.len() < 2 {
                        bail!("Line {line_num}: 'app' requires an app name");
                    }
                    let name = parts[1].to_string();
                    let mut global = false;
                    let mut hold = false;
                    let mut bucket = None;

                    let mut i = 2;
                    while i < parts.len() {
                        match parts[i] {
                            "--global" | "-g" => global = true,
                            "--hold" => hold = true,
                            "--bucket" | "-b" => {
                                if i + 1 < parts.len() {
                                    bucket = Some(parts[i + 1].to_string());
                                    i += 1;
                                }
                            }
                            arg => {
                                log::warn!("Line {line_num}: Unknown app option '{arg}', ignored");
                            }
                        }
                        i += 1;
                    }

                    apps.push(BundleApp {
                        name,
                        bucket,
                        global,
                        hold,
                    });
                }
                cmd => {
                    log::warn!("Line {line_num}: Unknown Hpfile directive '{cmd}', skipping");
                }
            }
        }

        Ok(Hpfile { buckets, apps })
    }

    /// Read an Hpfile from path
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read Hpfile at {}", path.as_ref().display()))?;
        Self::parse(&content)
    }

    /// Serialize Hpfile to a formatted string
    pub fn to_string_formatted(&self) -> String {
        let mut out = String::new();
        out.push_str("# Hpfile - Hyperscoop Bundle Configuration\n");
        out.push_str("# Generated automatically. Can be edited and committed.\n\n");

        if !self.buckets.is_empty() {
            out.push_str("# --- Buckets ---\n");
            for b in &self.buckets {
                if let Some(ref url) = b.url {
                    out.push_str(&format!("bucket {} {}\n", b.name, url));
                } else {
                    out.push_str(&format!("bucket {}\n", b.name));
                }
            }
            out.push('\n');
        }

        if !self.apps.is_empty() {
            out.push_str("# --- Applications ---\n");
            for a in &self.apps {
                let mut line = format!("app {}", a.name);
                if let Some(ref b) = a.bucket {
                    line.push_str(&format!(" --bucket {}", b));
                }
                if a.global {
                    line.push_str(" --global");
                }
                if a.hold {
                    line.push_str(" --hold");
                }
                line.push('\n');
                out.push_str(&line);
            }
        }

        out
    }

    /// Dump current environment into an `Hpfile`
    pub fn dump_current() -> anyhow::Result<Self> {
        let mut bundle_buckets = Vec::new();
        let buckets_obj = Buckets::new()?;

        let (urls, paths) = buckets_obj.get_bucket_source_url_and_path(false)?;
        for (url, path) in urls.into_iter().zip(paths.into_iter()) {
            let name = Path::new(&path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let url_opt = if url == "Unknown" || url.is_empty() {
                None
            } else {
                Some(url)
            };
            bundle_buckets.push(BundleBucket { name, url: url_opt });
        }

        let mut bundle_apps = Vec::new();
        let mut seen = HashSet::new();

        let scan_targets = [(get_apps_path(), false), (get_apps_path_global(), true)];

        for (dir, is_global) in scan_targets {
            let p = Path::new(&dir);
            if !p.is_dir() {
                continue;
            }
            let entries = match read_dir(p) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let app_name = entry.file_name().to_string_lossy().to_string();
                if app_name.eq_ignore_ascii_case("scoop") {
                    continue;
                }

                let install_json_path = if is_global {
                    get_app_dir_install_json_global(&app_name)
                } else {
                    get_app_dir_install_json(&app_name)
                };

                let install_path = Path::new(&install_json_path);
                if !install_path.is_file() {
                    continue;
                }

                let bucket = get_install_json_bucket(&install_json_path).ok();
                let hold = is_app_held_by_path(&install_json_path);

                let key = (app_name.to_lowercase(), is_global);
                if !seen.contains(&key) {
                    seen.insert(key);
                    bundle_apps.push(BundleApp {
                        name: app_name,
                        bucket,
                        global: is_global,
                        hold,
                    });
                }
            }
        }

        bundle_apps.sort_by(|a, b| a.name.cmp(&b.name));
        bundle_buckets.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Hpfile {
            buckets: bundle_buckets,
            apps: bundle_apps,
        })
    }
}

/// Result of checking an Hpfile
#[derive(Debug, Default)]
pub struct BundleCheckResult {
    pub satisfied_buckets: Vec<String>,
    pub missing_buckets: Vec<BundleBucket>,
    pub satisfied_apps: Vec<String>,
    pub missing_apps: Vec<BundleApp>,
}

impl BundleCheckResult {
    pub fn is_all_satisfied(&self) -> bool {
        self.missing_buckets.is_empty() && self.missing_apps.is_empty()
    }
}

/// Check if current system satisfies the given Hpfile
pub fn check_bundle(hpfile: &Hpfile) -> anyhow::Result<BundleCheckResult> {
    let mut result = BundleCheckResult::default();
    let buckets_obj = Buckets::new()?;
    let current_buckets: HashSet<String> = buckets_obj
        .buckets_name
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();

    for b in &hpfile.buckets {
        if current_buckets.contains(&b.name.to_lowercase()) {
            result.satisfied_buckets.push(b.name.clone());
        } else {
            result.missing_buckets.push(b.clone());
        }
    }

    for a in &hpfile.apps {
        let install_json = if a.global {
            get_app_dir_install_json_global(&a.name)
        } else {
            get_app_dir_install_json(&a.name)
        };

        if Path::new(&install_json).is_file() {
            result.satisfied_apps.push(a.name.clone());
        } else {
            result.missing_apps.push(a.clone());
        }
    }

    Ok(result)
}

/// Execute bundle install: restore buckets and apps from Hpfile
pub fn install_bundle(hpfile: &Hpfile) -> anyhow::Result<()> {
    let buckets_obj = Buckets::new()?;
    let current_buckets: HashSet<String> = buckets_obj
        .buckets_name
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    // 1. Add missing buckets
    println!(
        "{}",
        tr(
            "==> Checking and adding buckets...",
            "==> 正在检查并添加 Bucket..."
        )
        .dark_cyan()
        .bold()
    );

    for b in &hpfile.buckets {
        if current_buckets.contains(&b.name.to_lowercase()) {
            println!(
                "  {} {}",
                "✔".dark_green().bold(),
                tr_fmt!(
                    "Bucket '{name}' already present",
                    "Bucket '{name}' 已存在",
                    name = &b.name
                )
            );
        } else {
            println!(
                "  {} {}",
                "+".cyan().bold(),
                tr_fmt!(
                    "Adding bucket '{name}'...",
                    "正在添加 bucket '{name}'...",
                    name = &b.name
                )
            );
            if let Err(e) = buckets_obj.add_buckets(Some(b.name.clone()), b.url.clone(), false) {
                eprintln!(
                    "  {} {}",
                    "✖".dark_red().bold(),
                    tr_fmt!(
                        "Failed to add bucket '{name}': {e}",
                        "添加 bucket '{name}' 失败: {e}",
                        name = &b.name,
                        e = e
                    )
                );
            }
        }
    }

    // 2. Install missing apps
    println!(
        "{}",
        tr(
            "\n==> Checking and installing applications...",
            "\n==> 正在检查并安装应用程序..."
        )
        .dark_cyan()
        .bold()
    );

    for a in &hpfile.apps {
        let install_json = if a.global {
            get_app_dir_install_json_global(&a.name)
        } else {
            get_app_dir_install_json(&a.name)
        };

        if Path::new(&install_json).is_file() {
            println!(
                "  {} {}",
                "✔".dark_green().bold(),
                tr_fmt!(
                    "'{name}' is already installed",
                    "'{name}' 已安装",
                    name = &a.name
                )
            );
        } else {
            println!(
                "  {} {}",
                "⬇".dark_yellow().bold(),
                tr_fmt!(
                    "Installing '{name}'...",
                    "正在安装 '{name}'...",
                    name = &a.name
                )
            );

            let mut options = Vec::new();
            if a.global {
                options.push(InstallOptions::Global);
            }

            let install_res = if let Some(ref bucket_name) = a.bucket {
                install_from_specific_bucket(bucket_name, &a.name, &options)
            } else {
                install_app(&a.name, &options)
            };

            match install_res {
                Ok(_) => {
                    println!(
                        "  {} {}",
                        "✔".dark_green().bold(),
                        tr_fmt!(
                            "'{name}' installed successfully",
                            "'{name}' 安装成功",
                            name = &a.name
                        )
                    );
                    if a.hold {
                        set_app_hold(&a.name, a.global, true);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "  {} {}",
                        "✖".dark_red().bold(),
                        tr_fmt!(
                            "Failed to install '{name}': {e}",
                            "安装 '{name}' 失败: {e}",
                            name = &a.name,
                            e = e
                        )
                    );
                }
            }
        }
    }

    Ok(())
}

/// Helper to set hold status in install.json
pub fn set_app_hold(app_name: &str, is_global: bool, hold: bool) {
    let install_json = if is_global {
        get_app_dir_install_json_global(app_name)
    } else {
        let user = get_app_dir_install_json(app_name);
        if Path::new(&user).is_file() {
            user
        } else {
            get_app_dir_install_json_global(app_name)
        }
    };

    let p = Path::new(&install_json);
    if !p.is_file() {
        return;
    }

    if let Ok(content) = std::fs::read_to_string(p) {
        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(map) = value.as_object_mut() {
                if hold {
                    map.insert("hold".to_string(), serde_json::Value::Bool(true));
                } else {
                    map.remove("hold");
                }
                if let Ok(serialized) = serde_json::to_string_pretty(&value) {
                    let _ = std::fs::write(p, serialized);
                }
            }
        }
    }
}

/// Result of bundle cleanup
#[derive(Debug, Default)]
pub struct BundleCleanupResult {
    pub unlisted_apps: Vec<BundleApp>,
}

/// Check or perform bundle cleanup against an Hpfile
pub fn cleanup_bundle(hpfile: &Hpfile, force: bool) -> anyhow::Result<BundleCleanupResult> {
    let hpfile_apps: HashSet<(String, bool)> = hpfile
        .apps
        .iter()
        .map(|a| (a.name.to_lowercase(), a.global))
        .collect();

    let mut unlisted = Vec::new();
    let mut seen = HashSet::new();
    let scan_targets = [(get_apps_path(), false), (get_apps_path_global(), true)];

    for (dir, is_global) in scan_targets {
        let p = Path::new(&dir);
        if !p.is_dir() {
            continue;
        }
        let entries = match read_dir(p) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let app_name = entry.file_name().to_string_lossy().to_string();
            let app_lower = app_name.to_lowercase();
            // Protect system apps
            if app_lower == "scoop" || app_lower == "hp" {
                continue;
            }

            let install_json_path = if is_global {
                get_app_dir_install_json_global(&app_name)
            } else {
                get_app_dir_install_json(&app_name)
            };

            if !Path::new(&install_json_path).is_file() {
                continue;
            }

            let key = (app_lower.clone(), is_global);
            if !hpfile_apps.contains(&key) && !seen.contains(&key) {
                seen.insert(key);
                unlisted.push(BundleApp {
                    name: app_name,
                    bucket: get_install_json_bucket(&install_json_path).ok(),
                    global: is_global,
                    hold: is_app_held_by_path(&install_json_path),
                });
            }
        }
    }

    if force {
        for app in &unlisted {
            println!(
                "  {} {}",
                "-".red().bold(),
                tr_fmt!(
                    "Uninstalling unlisted app '{name}' (global={global})...",
                    "正在卸载未列出应用 '{name}' (global={global})...",
                    name = &app.name,
                    global = app.global
                )
            );
            let _ = crate::uninstall::uninstall_app(&app.name, app.global);
        }
    }

    Ok(BundleCleanupResult {
        unlisted_apps: unlisted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpfile_parse_and_format() {
        let input = r#"
# Test Hpfile
bucket main
bucket extras https://github.com/ScoopInstaller/Extras

app git
app 7zip --global
app neovim --bucket extras --hold
"#;

        let parsed = Hpfile::parse(input).unwrap();
        assert_eq!(parsed.buckets.len(), 2);
        assert_eq!(parsed.buckets[0].name, "main");
        assert_eq!(parsed.buckets[0].url, None);
        assert_eq!(parsed.buckets[1].name, "extras");
        assert_eq!(
            parsed.buckets[1].url.as_deref(),
            Some("https://github.com/ScoopInstaller/Extras")
        );

        assert_eq!(parsed.apps.len(), 3);
        assert_eq!(parsed.apps[0].name, "git");
        assert!(!parsed.apps[0].global);
        assert!(!parsed.apps[0].hold);

        assert_eq!(parsed.apps[1].name, "7zip");
        assert!(parsed.apps[1].global);

        assert_eq!(parsed.apps[2].name, "neovim");
        assert_eq!(parsed.apps[2].bucket.as_deref(), Some("extras"));
        assert!(parsed.apps[2].hold);

        let formatted = parsed.to_string_formatted();
        assert!(formatted.contains("bucket main"));
        assert!(formatted.contains("bucket extras https://github.com/ScoopInstaller/Extras"));
        assert!(formatted.contains("app git"));
        assert!(formatted.contains("app 7zip --global"));
        assert!(formatted.contains("app neovim --bucket extras --hold"));
    }

    #[test]
    fn test_hpfile_cleanup_diff() {
        let input = r#"
bucket main
app git
app curl
"#;
        let hpfile = Hpfile::parse(input).unwrap();
        // Installed: git, curl, neovim, wget
        let installed = vec![
            ("git".to_string(), false),
            ("curl".to_string(), false),
            ("neovim".to_string(), false),
            ("wget".to_string(), true),
        ];

        let to_remove: Vec<(String, bool)> = installed
            .into_iter()
            .filter(|(app, is_global)| {
                !hpfile
                    .apps
                    .iter()
                    .any(|a| a.name.eq_ignore_ascii_case(app) && a.global == *is_global)
            })
            .collect();

        assert_eq!(to_remove.len(), 2);
        assert_eq!(to_remove[0], ("neovim".to_string(), false));
        assert_eq!(to_remove[1], ("wget".to_string(), true));
    }
}
