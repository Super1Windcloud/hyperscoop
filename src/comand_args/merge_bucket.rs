
use  clap::Args;


#[derive(Args, Debug)]
///🐧          移除不同buckets中冗余的manifest文件
pub struct MergeArgs     {
  name: Option<String>,
}
