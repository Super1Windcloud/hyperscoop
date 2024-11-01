use clap::Subcommand;
pub(crate) use   crate::comand_args:: {bucket_args::BucketArgs, cache::CacheArgs} ;
use crate::comand_args::cat::CatArgs;
use crate::comand_args::checkup::checkupArgs;
use crate::comand_args::cleanup::CleanupArgs;
use crate::comand_args::config::ConfigArgs;
use crate::comand_args::export::ExportArgs;
use crate::comand_args::home::HomeArgs;
use crate::comand_args::import::ImportArgs;
use crate::comand_args::info::InfoArgs;
use crate::comand_args::install::InstallArgs;
use crate::comand_args::list::ListArgs;
use crate::comand_args::merge_bucket::MergeArgs;
use crate::comand_args::prefix::PrefixArgs;
use crate::comand_args::reset::ResetArgs;
use crate::comand_args::search::SearchArgs;
use crate::comand_args::shim::ShimArgs;
use crate::comand_args::status::StatusArgs;
use crate::comand_args::uninstall::UninstallArgs;
use crate::comand_args::update::UpdateArgs;
use crate::comand_args::which::WhichArgs;

#[derive(Subcommand ,Debug  )]
#[command(propagate_version = true)] // 自动传递版本信息
#[command(subcommand_negates_reqs = true)] // 禁止子命令的短选项冲突
#[command(infer_subcommands = true , infer_long_args = true )] // 自动推断子命令和长选项
#[command(arg_required_else_help = true  , next_line_help = true    )] //帮助信息换行
#[command(color_auto = true  , color_help = true    )] // 自动检测终端颜色
pub(crate) enum Commands {

  Bucket(BucketArgs) ,
  Cat (CatArgs ),
  Cache(CacheArgs),
  Checkup(checkupArgs),
  Cleanup(CleanupArgs),
  Config  (ConfigArgs),
  Export (ExportArgs),
  Home (HomeArgs) ,
  Import (ImportArgs),
  Info (InfoArgs),
  Install(InstallArgs),
  List (ListArgs),
  Prefix (PrefixArgs),
  Reset (ResetArgs),
  Search(SearchArgs),
  Shim (ShimArgs),
   Status (StatusArgs),
  Uninstall(UninstallArgs),
  Update (UpdateArgs),
  Which(WhichArgs ),
  Merge(MergeArgs )


}
