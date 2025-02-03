use clap::Args; 
#[derive(Args, Debug)]

#[clap(author, version, about="🐇          检查所有潜在问题,别名check", long_about = None)]
#[clap(alias = "check")]
pub struct CheckupArgs {
}
