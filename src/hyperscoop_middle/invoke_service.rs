use crate::command_args::service::{ServiceArgs, ServiceSubcommands};
use crate::i18n::tr;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
#[cfg(windows)]
use std::process::Command;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    Starting,
    Stopping,
    NotFound,
    Unknown(String),
}

impl ServiceState {
    pub fn display_colored(&self) -> String {
        match self {
            ServiceState::Running => format!("{}", tr("running", "运行中").green().bold()),
            ServiceState::Stopped => format!("{}", tr("stopped", "已停止").yellow()),
            ServiceState::Starting => format!("{}", tr("starting", "启动中").cyan()),
            ServiceState::Stopping => format!("{}", tr("stopping", "停止中").magenta()),
            ServiceState::NotFound => format!("{}", tr("not installed", "未注册服务").dark_grey()),
            ServiceState::Unknown(s) => format!("{}", s.as_str().dark_grey()),
        }
    }
}

pub fn execute_service_command(args: ServiceArgs) -> anyhow::Result<()> {
    let sub = args.command.unwrap_or(ServiceSubcommands::List);
    match sub {
        ServiceSubcommands::List => list_services()?,
        ServiceSubcommands::Status { name } => show_service_status(&name)?,
        ServiceSubcommands::Start { name } => start_service(&name)?,
        ServiceSubcommands::Stop { name } => stop_service(&name)?,
        ServiceSubcommands::Restart { name } => restart_service(&name)?,
    }
    Ok(())
}

#[cfg(windows)]
fn query_service_state(service_name: &str) -> ServiceState {
    let output = Command::new("sc.exe")
        .args(["query", service_name])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}\n{}", stdout, stderr);

            if combined.contains("1060") {
                return ServiceState::NotFound;
            }
            if combined.contains("STATE") {
                if combined.contains("RUNNING") {
                    return ServiceState::Running;
                } else if combined.contains("STOPPED") {
                    return ServiceState::Stopped;
                } else if combined.contains("START_PENDING") {
                    return ServiceState::Starting;
                } else if combined.contains("STOP_PENDING") {
                    return ServiceState::Stopping;
                }
            }
            ServiceState::NotFound
        }
        Err(_) => ServiceState::NotFound,
    }
}

#[cfg(not(windows))]
fn query_service_state(_service_name: &str) -> ServiceState {
    ServiceState::NotFound
}

fn show_service_status(name: &str) -> anyhow::Result<()> {
    let state = query_service_state(name);
    println!(
        "{} {}: {}",
        tr("Service", "服务").dark_cyan().bold(),
        name.bold(),
        state.display_colored()
    );
    Ok(())
}

#[allow(unused_variables)]
fn start_service(name: &str) -> anyhow::Result<()> {
    if !cfg!(windows) {
        println!(
            "{}",
            tr(
                "Service management is only supported on Windows.",
                "服务管理仅支持 Windows 系统。"
            )
            .yellow()
        );
        return Ok(());
    }

    #[cfg(windows)]
    {
        println!("{} '{}'...", tr("Starting service", "正在启动服务"), name);
        let output = Command::new("sc.exe").args(["start", name]).output()?;

        if output.status.success() {
            println!(
                "{}",
                tr(
                    &format!("Successfully started service '{}'.", name),
                    &format!("成功启动服务 '{}'。", name)
                )
                .green()
            );
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            let out = String::from_utf8_lossy(&output.stdout);
            eprintln!(
                "{}: {}\n{}",
                tr("Failed to start service", "启动服务失败")
                    .dark_red()
                    .bold(),
                out.trim(),
                err.trim()
            );
        }
    }
    Ok(())
}

#[allow(unused_variables)]
fn stop_service(name: &str) -> anyhow::Result<()> {
    if !cfg!(windows) {
        println!(
            "{}",
            tr(
                "Service management is only supported on Windows.",
                "服务管理仅支持 Windows 系统。"
            )
            .yellow()
        );
        return Ok(());
    }

    #[cfg(windows)]
    {
        println!("{} '{}'...", tr("Stopping service", "正在停止服务"), name);
        let output = Command::new("sc.exe").args(["stop", name]).output()?;

        if output.status.success() {
            println!(
                "{}",
                tr(
                    &format!("Successfully stopped service '{}'.", name),
                    &format!("成功停止服务 '{}'。", name)
                )
                .green()
            );
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            let out = String::from_utf8_lossy(&output.stdout);
            eprintln!(
                "{}: {}\n{}",
                tr("Failed to stop service", "停止服务失败")
                    .dark_red()
                    .bold(),
                out.trim(),
                err.trim()
            );
        }
    }
    Ok(())
}

fn restart_service(name: &str) -> anyhow::Result<()> {
    stop_service(name)?;
    std::thread::sleep(std::time::Duration::from_millis(800));
    start_service(name)?;
    Ok(())
}

fn list_services() -> anyhow::Result<()> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new(tr("Name", "应用/服务名"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new(tr("Status", "状态"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new(tr("Type", "类型"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]);

    // Candidates: installed apps or common developer services
    let mut candidates = vec![
        "redis".to_string(),
        "mysql".to_string(),
        "mariadb".to_string(),
        "postgresql".to_string(),
        "nginx".to_string(),
        "caddy".to_string(),
        "mongodb".to_string(),
        "memcached".to_string(),
        "ollama".to_string(),
        "docker".to_string(),
    ];

    if let Ok(installed) = command_util_lib::list::list_all_installed_apps_refactor(false) {
        for app in installed {
            if !candidates.contains(&app.name) {
                candidates.push(app.name);
            }
        }
    }

    let mut found_any = false;
    for name in &candidates {
        let state = query_service_state(name);
        if state != ServiceState::NotFound {
            found_any = true;
            let status_cell = match state {
                ServiceState::Running => Cell::new(tr("running", "运行中")).fg(Color::Green),
                ServiceState::Stopped => Cell::new(tr("stopped", "已停止")).fg(Color::Yellow),
                ServiceState::Starting => Cell::new(tr("starting", "启动中")).fg(Color::Cyan),
                ServiceState::Stopping => Cell::new(tr("stopping", "停止中")).fg(Color::Magenta),
                _ => Cell::new("unknown"),
            };
            table.add_row(vec![
                Cell::new(name).add_attribute(Attribute::Bold),
                status_cell,
                Cell::new("Windows Service"),
            ]);
        }
    }

    if found_any {
        println!("{table}");
    } else {
        println!(
            "{}",
            tr(
                "No active services found for installed packages.",
                "未发现已安装应用相关的活跃后台服务。"
            )
            .dark_grey()
        );
        println!(
            "{}",
            tr(
                "You can register and manage background services with: hp service start/stop/status <name>",
                "您可以通过 hp service start/stop/status <name> 启动、停止或查询服务状态。"
            )
            .cyan()
        );
    }

    Ok(())
}
