use clap::Args;


#[derive(Args, Debug)]
///🍹          更新指定APP或者hyperscoop和buckets
pub struct UpdateArgs {
  #[arg(required = false)]
  #[clap(help = "指定要更新的APP名称,如果没有name参数,则更新scoop和buckets,\
  只对官方维护的bucket进行更新")]
  name: Option<String>,
}
