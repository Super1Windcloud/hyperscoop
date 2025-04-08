use clap::Args;
#[derive(Args, Debug)]

#[clap(author, version, about="🐇\t\t检查所有潜在问题,别名check", long_about = None)]
#[clap(alias = "check")]
pub struct CheckupArgs {
  #[arg(from_global)]
  pub global : bool,
}
