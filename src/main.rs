
use std::{env, process};
use log::{debug};
mod comand_args ;
use clap_verbosity_flag ;
use crossterm::style::{Color, PrintStyledContent, Stylize};
use std::path::PathBuf;
use clap::{Parser, Subcommand    ,command  , Command      };
mod command;
use command::Commands;
#[derive(Parser ,Debug )]
#[command(name="hyperscoop" , version, about= None , long_about = None)]
#[command(propagate_version = true  )]  //  版本信息传递
#[command(override_usage  = "hyperscoop [COMMAND]  [OPTIONS] ")]
#[command(author = "superwindcloud")]
#[command(after_help = "For more information about a command, run: hyperscoop COMMAND -h/--help")]
struct Cli {

  #[command(subcommand)]
  command: Option<Commands>,

  #[command(flatten)]
  verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
  println!( "{ }!   \n " , "次世代更快更强更精美的Windows包管理器".magenta().bold()) ;
  let cli = Cli::parse();

  if  cli.command.is_none() {

  }



}


