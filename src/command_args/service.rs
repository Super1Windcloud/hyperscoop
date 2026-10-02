use clap::{Args, Subcommand};

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("⚙️\t\tManage background services (similar to brew services)", "⚙️\t\t管理后台服务 (类似 brew services)"),
    long_about = None
)]
#[command(arg_required_else_help = false)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub command: Option<ServiceSubcommands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ServiceSubcommands {
    #[clap(about = crate::i18n::tr("List all background services and their statuses", "列出所有后台服务及其运行状态"))]
    List,
    #[clap(about = crate::i18n::tr("Show status of a specific service", "查看指定服务的运行状态"))]
    Status {
        #[arg(help = crate::i18n::tr("Service or app name", "服务或应用名称"))]
        name: String,
    },
    #[clap(about = crate::i18n::tr("Start a service", "启动指定服务"))]
    Start {
        #[arg(help = crate::i18n::tr("Service or app name to start", "要启动的服务或应用名称"))]
        name: String,
    },
    #[clap(about = crate::i18n::tr("Stop a service", "停止指定服务"))]
    Stop {
        #[arg(help = crate::i18n::tr("Service or app name to stop", "要停止的服务或应用名称"))]
        name: String,
    },
    #[clap(about = crate::i18n::tr("Restart a service", "重启指定服务"))]
    Restart {
        #[arg(help = crate::i18n::tr("Service or app name to restart", "要重启的服务或应用名称"))]
        name: String,
    },
}
