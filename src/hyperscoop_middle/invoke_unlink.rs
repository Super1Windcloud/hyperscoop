use crate::command_args::unlink::UnlinkArgs;
use crate::i18n::tr_fmt;
use anyhow::{Context, bail};
use command_util_lib::init_env::{
    get_app_dir_manifest_json, get_app_dir_manifest_json_global, get_shims_root_dir,
    get_shims_root_dir_global,
};
use command_util_lib::manifest::uninstall_manifest::UninstallManifest;
use command_util_lib::uninstall::shim_and_shortcuts::{rm_shim_file, rm_start_menu_shortcut};
use crossterm::style::Stylize;
use std::path::Path;

pub fn execute_unlink_command(args: UnlinkArgs) -> anyhow::Result<()> {
    for app_name in &args.app_names {
        let manifest_path = if args.global {
            get_app_dir_manifest_json_global(app_name)
        } else {
            get_app_dir_manifest_json(app_name)
        };

        if !Path::new(&manifest_path).exists() {
            bail!(
                "{}",
                tr_fmt!(
                    "App '{name}' manifest not found. Is it installed?",
                    "未找到应用 '{name}' 的 manifest，请确认是否已安装？",
                    name = app_name
                )
            );
        }

        let content = std::fs::read_to_string(&manifest_path)
            .context(format!("Failed to read manifest for {}", app_name))?;
        let manifest: UninstallManifest = serde_json::from_str(&content)
            .context(format!("Failed to parse manifest for {}", app_name))?;

        let shim_path = if args.global {
            get_shims_root_dir_global()
        } else {
            get_shims_root_dir()
        };

        rm_shim_file(&shim_path, &manifest, app_name)?;
        let _ = rm_start_menu_shortcut(&manifest, args.global);

        println!(
            "{}",
            tr_fmt!(
                "Successfully unlinked '{name}'. Shims and shortcuts removed.",
                "已成功断开 '{name}' 链接，对应 shim 与快捷方式已移除。",
                name = app_name
            )
            .yellow()
            .bold()
        );
    }
    Ok(())
}
