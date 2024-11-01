use  clap::Args;


#[derive(Args, Debug)]
///🐇          检查所有潜在问题
pub struct checkupArgs  {
  name: Option<String>,
}
