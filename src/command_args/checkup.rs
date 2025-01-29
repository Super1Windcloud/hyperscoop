use clap::Args; 
#[derive(Args, Debug)]
///🐇          检查所有潜在问题 
#[clap(author, version, about="检查所有潜在问题, 别名check", long_about = None)]
#[clap(alias = "check")]
pub struct CheckupArgs {
}
