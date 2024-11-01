
use  clap::Args;


#[derive(Args, Debug)]
///🍹          更新指定APP或者scoop自身和buckets
pub struct  UpdateArgs            {
  name: Option<String>,
}
