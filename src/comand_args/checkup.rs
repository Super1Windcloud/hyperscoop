use clap::Args;

#[derive(Args, Debug)]
///🐇          检查所有潜在问题
pub struct CheckupArgs {
  name: Option<String>,
}
