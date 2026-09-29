use crate::buckets::get_buckets_path;
use crate::init_env::{
    get_app_dir_install_json, get_app_dir_install_json_global, get_app_dir_manifest_json,
    get_app_dir_manifest_json_global, get_apps_path, get_apps_path_global,
};
use crate::list::{get_install_json_bucket, get_install_json_version};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest::get_best_manifest_from_local_bucket;
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::read_dir;
use std::path::Path;

/// Normalize a dependency name, stripping bucket prefixes (e.g. "main/git" -> "git")
pub fn normalize_dep_name(dep: &str) -> String {
    let trimmed = dep.trim();
    if let Some((_bucket, app)) = trimmed.split_once('/') {
        app.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Extract all declared dependencies from an `InstallManifest`
pub fn get_manifest_dependencies(manifest: &InstallManifest) -> Vec<String> {
    match &manifest.depends {
        Some(StringArrayOrString::String(s)) => {
            if s.trim().is_empty() {
                Vec::new()
            } else {
                vec![s.clone()]
            }
        }
        Some(StringArrayOrString::StringArray(arr)) => arr
            .iter()
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .collect(),
        _ => Vec::new(),
    }
}

/// Information about an application that uses a given target application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDependencyUsage {
    pub app_name: String,
    pub declared_dep: String,
    pub is_global: bool,
    pub bucket: String,
    pub version: String,
}

/// Find all installed apps (user and/or global) that declare `target_app` in their `depends`
pub fn find_installed_dependents(
    target_app: &str,
    global_only: bool,
) -> anyhow::Result<Vec<AppDependencyUsage>> {
    let target_norm = normalize_dep_name(target_app).to_lowercase();
    let mut results = Vec::new();

    let scan_dirs = if global_only {
        vec![(get_apps_path_global(), true)]
    } else {
        vec![(get_apps_path(), false), (get_apps_path_global(), true)]
    };

    for (root_dir, is_global) in scan_dirs {
        let path = Path::new(&root_dir);
        if !path.is_dir() {
            continue;
        }
        let entries = match read_dir(path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let app_name = entry.file_name().to_string_lossy().to_string();
            if app_name.eq_ignore_ascii_case("scoop") || app_name.eq_ignore_ascii_case(target_app) {
                continue;
            }

            let manifest_path = if is_global {
                get_app_dir_manifest_json_global(&app_name)
            } else {
                get_app_dir_manifest_json(&app_name)
            };

            let m_path = Path::new(&manifest_path);
            if !m_path.is_file() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(m_path) {
                if let Ok(manifest) = serde_json::from_str::<InstallManifest>(&content) {
                    let deps = get_manifest_dependencies(&manifest);
                    for dep in deps {
                        if normalize_dep_name(&dep).to_lowercase() == target_norm {
                            let install_json = if is_global {
                                get_app_dir_install_json_global(&app_name)
                            } else {
                                get_app_dir_install_json(&app_name)
                            };
                            let bucket = get_install_json_bucket(&install_json)
                                .unwrap_or_else(|_| "unknown".to_string());
                            let version = get_install_json_version(&manifest_path)
                                .unwrap_or_else(|_| "unknown".to_string());

                            results.push(AppDependencyUsage {
                                app_name: app_name.clone(),
                                declared_dep: dep,
                                is_global,
                                bucket,
                                version,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| a.app_name.cmp(&b.app_name));
    results.dedup_by(|a, b| a.app_name == b.app_name && a.is_global == b.is_global);
    Ok(results)
}

/// Find all manifests in buckets that declare `target_app` in `depends`
pub fn find_bucket_dependents(target_app: &str) -> anyhow::Result<Vec<(String, String)>> {
    let target_norm = normalize_dep_name(target_app).to_lowercase();
    let buckets = get_buckets_path()?;

    let results: Vec<(String, String)> = buckets
        .par_iter()
        .flat_map(|b_path| {
            let p = Path::new(b_path);
            let bucket_name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let manifests_dir = p.join("bucket");
            let target_dir = if manifests_dir.is_dir() {
                manifests_dir
            } else {
                p.to_path_buf()
            };

            let Ok(read) = read_dir(&target_dir) else {
                return Vec::new();
            };

            read.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
                .filter_map(|e| {
                    let app_stem = e.path().file_stem()?.to_string_lossy().to_string();
                    if app_stem.eq_ignore_ascii_case(target_app) {
                        return None;
                    }
                    let content = std::fs::read_to_string(e.path()).ok()?;
                    let manifest: InstallManifest = serde_json::from_str(&content).ok()?;
                    let deps = get_manifest_dependencies(&manifest);
                    for dep in deps {
                        if normalize_dep_name(&dep).to_lowercase() == target_norm {
                            return Some((format!("{}/{}", bucket_name, app_stem), dep));
                        }
                    }
                    None
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(results)
}

/// Find manifest for an app, either from local installed files or from available buckets
pub fn find_manifest_for_app(
    app_name: &str,
    is_global: bool,
) -> anyhow::Result<(InstallManifest, String)> {
    // 1. Check local installed
    let installed_manifest_path = if is_global {
        get_app_dir_manifest_json_global(app_name)
    } else {
        let user_path = get_app_dir_manifest_json(app_name);
        if Path::new(&user_path).is_file() {
            user_path
        } else {
            get_app_dir_manifest_json_global(app_name)
        }
    };

    if Path::new(&installed_manifest_path).is_file() {
        let content = std::fs::read_to_string(&installed_manifest_path)?;
        let manifest: InstallManifest = serde_json::from_str(&content)?;
        return Ok((manifest, installed_manifest_path));
    }

    // 2. Search buckets
    let best_path = get_best_manifest_from_local_bucket(app_name)?;
    let content = std::fs::read_to_string(&best_path)?;
    let manifest: InstallManifest = serde_json::from_str(&content)?;
    Ok((manifest, best_path.to_string_lossy().to_string()))
}

/// A node in a dependency tree
#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub declared_as: String,
    pub is_circular: bool,
    pub children: Vec<DependencyNode>,
}

/// Build a recursive dependency tree for an application
pub fn build_dependency_tree(
    app_name: &str,
    is_global: bool,
    visited: &mut HashSet<String>,
) -> DependencyNode {
    let norm_name = normalize_dep_name(app_name).to_lowercase();
    let is_circular = visited.contains(&norm_name);

    let mut node = DependencyNode {
        name: normalize_dep_name(app_name),
        declared_as: app_name.to_string(),
        is_circular,
        children: Vec::new(),
    };

    if is_circular {
        return node;
    }

    visited.insert(norm_name.clone());

    if let Ok((manifest, _)) = find_manifest_for_app(app_name, is_global) {
        let deps = get_manifest_dependencies(&manifest);
        for dep in deps {
            let child_node = build_dependency_tree(&dep, is_global, visited);
            node.children.push(child_node);
        }
    }

    visited.remove(&norm_name);
    node
}

/// Print the dependency tree to stdout
pub fn print_dependency_tree(node: &DependencyNode, prefix: &str, is_last: bool, is_root: bool) {
    if is_root {
        println!("{}", node.name.clone().cyan().bold());
    } else {
        let branch = if is_last { "└── " } else { "├── " };
        let name_display = if node.is_circular {
            format!("{} (circular)", node.declared_as)
                .dark_yellow()
                .to_string()
        } else {
            node.declared_as.clone().white().to_string()
        };
        println!(
            "{}{}{}",
            prefix.dark_grey(),
            branch.dark_grey(),
            name_display
        );
    }

    let child_prefix = if is_root {
        ""
    } else if is_last {
        "    "
    } else {
        "│   "
    };
    let new_prefix = format!("{}{}", prefix, child_prefix);

    let count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let last = i + 1 == count;
        print_dependency_tree(child, &new_prefix, last, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_dep_name() {
        assert_eq!(normalize_dep_name("git"), "git");
        assert_eq!(normalize_dep_name("main/git"), "git");
        assert_eq!(normalize_dep_name("extras/7zip"), "7zip");
        assert_eq!(
            normalize_dep_name("  extras/vcredist2022  "),
            "vcredist2022"
        );
    }

    #[test]
    fn test_get_manifest_dependencies() {
        let mut manifest = InstallManifest::default();
        assert!(get_manifest_dependencies(&manifest).is_empty());

        manifest.depends = Some(StringArrayOrString::String("git".to_string()));
        assert_eq!(get_manifest_dependencies(&manifest), vec!["git"]);

        manifest.depends = Some(StringArrayOrString::StringArray(vec![
            "git".to_string(),
            "main/7zip".to_string(),
        ]));
        assert_eq!(
            get_manifest_dependencies(&manifest),
            vec!["git", "main/7zip"]
        );
    }
}
