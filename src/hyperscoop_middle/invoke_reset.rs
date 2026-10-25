use crate::command_args::reset::ResetArgs;
use anyhow::bail;
use command_util_lib::reset::*;
use crossterm::style::Stylize;

fn parse_reset_target(name: &str) -> anyhow::Result<(&str, Option<&str>)> {
    command_util_lib::info::validate_app_name(name)?;
    if name.contains('/') {
        bail!("Invalid app name for reset: {}", name);
    }
    if let Some((app_name, version)) = name.split_once('@') {
        if app_name.is_empty() || version.is_empty() || version.contains('@') {
            bail!("Invalid reset target: {}", name);
        }
        command_util_lib::utils::utility::validate_version(version)?;
        Ok((app_name, Some(version)))
    } else {
        Ok((name, None))
    }
}

pub fn execute_reset_command(args: ResetArgs) -> Result<(), anyhow::Error> {
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
        let res = parse_reset_target(name).and_then(|(app_name, version)| match version {
            Some(version) => {
                reset_specific_version(app_name, version, args.global, args.shim_reset)
            }
            None => reset_latest_version(app_name, args.global, args.shim_reset),
        });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scoop_names_and_versions() -> anyhow::Result<()> {
        assert_eq!(
            parse_reset_target("notepad++@8.8.0")?,
            ("notepad++", Some("8.8.0"))
        );
        assert_eq!(
            parse_reset_target("node.js@22.0.0")?,
            ("node.js", Some("22.0.0"))
        );
        assert_eq!(parse_reset_target("clash-verge")?, ("clash-verge", None));
        assert_eq!(
            parse_reset_target("app@1.0+build")?,
            ("app", Some("1.0+build"))
        );
        for name in [
            "../app",
            "main/app",
            "app@",
            "@1.0",
            "app@1@2",
            "app/../../other",
        ] {
            assert!(parse_reset_target(name).is_err(), "{name}");
        }
        Ok(())
    }
}
