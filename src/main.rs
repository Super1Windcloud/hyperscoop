use log::debug;
use std::{env, process};
mod comand_args;
use clap::{command, Args, Command, Parser, Subcommand};
use clap_verbosity_flag;
use crossterm::style::{Color, PrintStyledContent, Stylize};
use std::path::PathBuf;
mod command;
use crate::hyperscoop_middle::execute_bucket_command;
use command::Commands;

mod hyperscoop_middle;
mod logger_err;
use logger_err::init_logger;
#[derive(Parser, Debug)]
#[command(name="hyperscoop" , version, about= None , long_about = None)]
#[command(propagate_version = true)] //  版本信息传递
#[command(override_usage = "hyperscoop [COMMAND]  [OPTIONS] ")]
#[command(author = "superwindcloud")]
#[command(after_help = "For more information about a command, run: hyperscoop COMMAND -h/--help")]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,

  #[command(flatten)]
  verbose: clap_verbosity_flag::Verbosity,
}
#[tokio::main]  // 异步运行 main 函数
#[allow(unused_variables)]
#[allow(unused)]
async fn main() -> Result<(), anyhow::Error> {
  init_logger();
  println!(
    "{ }!   \n ",
    "次世代更快更强更精美的Windows包管理器".magenta().bold()
  );
  let cli = Cli::parse();
  debug!("Running command: {:?}", cli.command);
  match cli.command {
    None => {
      eprintln!("No command provided. Run `hyperscoop --help` to see available commands.");
      return Ok(());
    }
    Some(input_command) => {
      return match input_command {
        Commands::Bucket(bucket) => execute_bucket_command(&bucket.command).await,
        Commands::Cat(_) => return Ok(()),
        Commands::Cache(_) => return Ok(()),
        Commands::Checkup(_) => return Ok(()),
        Commands::Cleanup(_) => return Ok(()),
        Commands::Config(_) => return Ok(()),
        Commands::Export(_) => return Ok(()),
        Commands::Home(_) => return Ok(()),
        Commands::Import(_) => return Ok(()),
        Commands::Info(_) => return Ok(()),
        Commands::Install(_) => return Ok(()),
        Commands::List(_) => return Ok(()),
        Commands::Prefix(_) => return Ok(()),
        Commands::Reset(_) => return Ok(()),
        Commands::Search(_) => return Ok(()),
        Commands::Shim(_) => return Ok(()),
        Commands::Status(_) => return Ok(()),
        Commands::Uninstall(_) => return Ok(()),
        Commands::Update(_) => return Ok(()),
        Commands::Which(_) => return Ok(()),
        Commands::Merge(_) => return Ok(()),
        _ => {
          eprintln!(
            "No command provided. Run `hyperscoop --help` to see available commands."
          );
          return Err(anyhow::anyhow!("No command provided.")); // 返回一
        }
      };
    }
  }
  Ok(())
}
