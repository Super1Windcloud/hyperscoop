use clap::Args;


#[derive(Args, Debug)]
///🍹          更新指定APP或者hyperscoop和buckets
pub struct UpdateArgs {
  name: Option<String>,
}
