use crate::command_args::reset::ResetArgs;
use anyhow::bail;
use command_util_lib::reset::*;
use crossterm::style::Stylize;
use regex::Regex;

pub fn execute_reset_command(args: ResetArgs) -> Result<(), anyhow::Error> {
    let pattern = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]*@[a-zA-Z0-9][a-zA-Z0-9_.-]*$")?;
    let total = args.names.len();
    let mut errors = Vec::new();

    for (idx, name) in args.names.iter().enumerate() {
        if total > 1 {
            println!(
                "{}",
                format!("\n[{}/{}] Resetting '{}'...", idx + 1, total, name)
                    .dark_cyan()
                    .bold()
            );
        }
        let res = if name.contains('@') {
            let count = name.matches('@').count();
            if count != 1 {
                Err(anyhow::anyhow!(
                    "Invalid app name: {}, only allow one '@'",
                    name
                ))
            } else if !pattern.is_match(name) {
                Err(anyhow::anyhow!("Invalid app name: {}", name))
            } else {
                let mut parts = name.split('@');
                let app_name = parts.next().unwrap_or("");
                let app_version = parts.next().unwrap_or("");
                if app_name.is_empty() || app_version.is_empty() {
                    Err(anyhow::anyhow!(
                        "Invalid app name: {}, please provide app version or app name",
                        name
                    ))
                } else {
                    reset_specific_version(app_name, app_version, args.global, args.shim_reset)
                }
            }
        } else {
            reset_latest_version(name.as_str(), args.global, args.shim_reset)
        };

        if let Err(e) = res {
            eprintln!("{} {}: {:#}", "ERROR".dark_red().bold(), name, e);
            errors.push(format!("{}: {:#}", name, e));
        }
    }

    if !errors.is_empty() {
        bail!(
            "Failed to reset {} app(s):\n{}",
            errors.len(),
            errors.join("\n")
        );
    }

    Ok(())
}
