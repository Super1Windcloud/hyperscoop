use crate::command_args::shellenv::ShellenvArgs;
use command_util_lib::init_env::{get_apps_path, get_apps_path_global, get_shims_root_dir};
use std::env;

pub fn execute_shellenv_command(args: ShellenvArgs) -> anyhow::Result<()> {
    let target_shell = args.shell.unwrap_or_else(|| {
        if let Ok(shell) = env::var("SHELL") {
            let lower = shell.to_lowercase();
            if lower.contains("zsh") {
                "zsh".to_string()
            } else if lower.contains("bash") {
                "bash".to_string()
            } else if lower.contains("nu") {
                "nu".to_string()
            } else {
                "bash".to_string()
            }
        } else {
            "powershell".to_string()
        }
    });

    let shims_dir = get_shims_root_dir();
    let scoop_apps = get_apps_path();
    let scoop_root = std::path::Path::new(&scoop_apps)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "C:\\Users\\Default\\scoop".to_string());

    let global_apps = get_apps_path_global();
    let global_root = std::path::Path::new(&global_apps)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "C:\\ProgramData\\scoop".to_string());
    let global_shims = format!("{}\\shims", global_root);

    match target_shell.to_lowercase().as_str() {
        "powershell" | "pwsh" => {
            println!("# HyperScoop environment configuration for PowerShell");
            println!("$env:SCOOP = \"{scoop_root}\"");
            println!("$env:SCOOP_GLOBAL = \"{global_root}\"");
            println!(
                "if ($env:Path -notlike \"*{shims_dir}*\") {{ $env:Path = \"{shims_dir};{global_shims};$env:Path\" }}"
            );
        }
        "cmd" => {
            println!("@REM HyperScoop environment configuration for CMD");
            println!("set \"SCOOP={scoop_root}\"");
            println!("set \"SCOOP_GLOBAL={global_root}\"");
            println!("set \"PATH={shims_dir};{global_shims};%PATH%\"");
        }
        "nu" | "nushell" => {
            println!("# HyperScoop environment configuration for Nushell");
            println!("$env.SCOOP = \"{scoop_root}\"");
            println!("$env.SCOOP_GLOBAL = \"{global_root}\"");
            println!(
                "$env.PATH = ($env.PATH | prepend \"{shims_dir}\" | prepend \"{global_shims}\")"
            );
        }
        _ => {
            println!("# HyperScoop environment configuration for POSIX shell (Bash/Zsh)");
            // Convert Windows path to MSYS/Cygwin style if needed
            let posix_shims = shims_dir.replace('\\', "/");
            let posix_root = scoop_root.replace('\\', "/");
            println!("export SCOOP=\"{posix_root}\"");
            println!("export SCOOP_GLOBAL=\"{}\"", global_root.replace('\\', "/"));
            println!(
                "export PATH=\"{posix_shims}:{}:$PATH\"",
                global_shims.replace('\\', "/")
            );
        }
    }

    Ok(())
}
