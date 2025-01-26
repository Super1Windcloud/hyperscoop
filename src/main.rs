#![deny(clippy::shadow)]
mod command_args;
use clap::{command, Parser};
use clap_verbosity_flag;
use crossterm::style::Stylize;
mod command;
mod hyperscoop_middle;
use command::Commands;
use hyperscoop_middle::*;
mod logger_err;
use logger_err::init_logger;

#[derive(Parser, Debug)]
#[command(name="hp" , version, about= None , long_about = None)]
#[command(propagate_version = true)] //  版本信息传递
#[command(override_usage = "hp  [COMMAND]  [OPTIONS] ")]
#[command(author = "superwindcloud")]
#[command(after_help = "For more i nformation about a command, run: hp  COMMAND -h/--help")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}
#[tokio::main] // 异步运行 main 函数
#[allow(unused_variables)]
#[allow(unused)]
#[allow(unreachable_code, unreachable_patterns)]
async fn main() -> Result<(), anyhow::Error> {
    init_logger();
    println!(
        "{ }!   \n ",
        "次世代更快更强更精美的Windows包管理器".magenta().bold()
    );
    let cli = Cli::parse();
    return match cli.command {
        None => {
            eprintln!("No command provided. Run `hp  --help` to see available commands.");
            Ok(())
        }
        Some(input_command) => {
            match input_command {
                Commands::Bucket(bucket) => execute_bucket_command(&bucket.command).await,
                Commands::Cat(cat) => execute_cat_command(cat),
                Commands::Cache(_) => return Ok(()),
                Commands::Checkup(_) => return Ok(()),
                Commands::Cleanup(_) => return Ok(()),
                Commands::Config(_) => return Ok(()),
                Commands::Export(_) => return Ok(()),
                Commands::Home(home) => execute_home_command(home),
                Commands::Import(_) => return Ok(()),
                Commands::Info(info) => execute_info_command(info),
                Commands::Install(_) => return Ok(()),
                Commands::List(query_app) => execute_list_installed_apps(query_app.name.as_ref()),
                Commands::Prefix( prefix ) =>  execute_prefix_command( prefix ), 
                Commands::Reset(_) => return Ok(()),
                Commands::Search(search_app) => execute_search_command(search_app),
                Commands::Shim(_) => return Ok(()),
                Commands::Status(_) => return Ok(()),
                Commands::Uninstall(_) => return Ok(()),
                Commands::Update(update_args) => execute_update_command(update_args),
                Commands::Which(_) => return Ok(()),
                Commands::Merge(_) => execute_merge_command(),
                _ => {
                    eprintln!("No command provided. Run `hp  --help` to see available commands.");
                    return Err(anyhow::anyhow!("No command provided.")); // 返回一
                }
            }
        }
    };
    Ok(())
}
