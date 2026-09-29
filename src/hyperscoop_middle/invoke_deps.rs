use crate::command_args::deps::DepsArgs;
use crate::i18n::{tr, tr_fmt};
use command_util_lib::depends::{
    build_dependency_tree, find_manifest_for_app, get_manifest_dependencies, normalize_dep_name,
    print_dependency_tree,
};
use command_util_lib::init_env::{get_app_dir_manifest_json, get_app_dir_manifest_json_global};
use crossterm::style::Stylize;
use std::collections::HashSet;
use std::path::Path;

pub fn execute_deps_command(args: DepsArgs) -> Result<(), anyhow::Error> {
    let app_name = &args.app_name;

    if args.tree {
        println!(
            "{}",
            tr_fmt!(
                "Dependency tree for '{name}':",
                "'{name}' 的依赖关系树：",
                name = app_name
            )
            .dark_cyan()
            .bold()
        );

        let mut visited = HashSet::new();
        let tree = build_dependency_tree(app_name, args.global, &mut visited);
        if tree.children.is_empty() {
            println!(
                "{}",
                tr_fmt!(
                    "'{name}' has no dependencies.",
                    "'{name}' 没有声明任何依赖项。",
                    name = app_name
                )
                .dark_grey()
            );
        } else {
            print_dependency_tree(&tree, "", true, true);
        }
    } else {
        let (manifest, path) = find_manifest_for_app(app_name, args.global)?;
        let deps = get_manifest_dependencies(&manifest);

        println!(
            "{} {} ({})",
            tr("Dependencies for", "应用程序依赖:").dark_cyan().bold(),
            app_name.as_str().cyan().bold(),
            path.dark_grey()
        );

        if deps.is_empty() {
            println!(
                "  {}",
                tr_fmt!(
                    "No dependencies required for '{name}'.",
                    "'{name}' 无需任何依赖。",
                    name = app_name
                )
                .dark_grey()
            );
            return Ok(());
        }

        for dep in &deps {
            let norm = normalize_dep_name(dep);
            let user_p = get_app_dir_manifest_json(&norm);
            let global_p = get_app_dir_manifest_json_global(&norm);
            let is_installed = Path::new(&user_p).is_file() || Path::new(&global_p).is_file();

            let status_badge = if is_installed {
                format!("[{}]", tr("Installed", "已安装")).dark_green()
            } else {
                format!("[{}]", tr("Not Installed", "未安装")).dark_yellow()
            };

            println!("  • {:<20} {}", dep.as_str().white().bold(), status_badge);
        }

        println!(
            "\n{}",
            tr_fmt!(
                "Total: {count} direct dependenc(ies).",
                "共计 {count} 个直接依赖项。",
                count = deps.len()
            )
            .dark_cyan()
        );
    }

    Ok(())
}
