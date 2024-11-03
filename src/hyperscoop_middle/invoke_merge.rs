use crossterm::style::Stylize;
use anyhow;
use command_util_lib::merge;
#[allow(unused)]
pub fn merge_all_bucket() {}

pub fn execute_merge_command() -> Result<(), anyhow::Error> {
  let result = merge::merge_all_buckets();
  if result.is_err() { println!("{} {:?}", "Error: {:?}".red().bold(), result); }
  Ok(())
}
