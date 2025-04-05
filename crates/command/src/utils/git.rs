use git2::{BranchType, ProxyOptions};
use std::path::Path;
use anyhow::bail;
use git2::{FetchOptions, Progress, Remote, RemoteCallbacks, Repository};
use gix::{
   remote::ref_map,
  ObjectId
};
use serde::Deserialize;
use crate::config::get_config_value_no_print;
use crate::utils::pull::run_pull;

mod errors {
  //! Git specific error helpers

  #[derive(Debug, thiserror::Error)]
  #[allow(missing_docs)]
  /// A collection of gitoxide errors
  pub enum GitoxideError {
    #[error("Gitoxide error: {0}")]
    GitoxideOpen(#[from] gix::open::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideTraverse(#[from] gix::traverse::commit::simple::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRevWalk(#[from] gix::revision::walk::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideHead(#[from] gix::reference::head_commit::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideDecode(#[from] gix_object::decode::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRevWalkGraph(#[from] gix::object::find::existing::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideCommit(#[from] gix::object::commit::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRewrites(#[from] gix::diff::new_rewrites::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideObjectPeel(#[from] gix::object::peel::to_kind::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideObjectDiff(#[from] gix::object::tree::diff::for_each::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideFindExisting(#[from] gix::reference::find::existing::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRemoteConnection(#[from] gix::remote::connect::Error),
    #[error("Gitoxide error: {0}")]
    GitoxidePeelCommit(#[from] gix::head::peel::to_commit::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRefMap(#[from] gix::remote::ref_map::Error),
    #[error("Gitoxide error: {0}")]
    GitoxidePrepareFetch(#[from] gix::remote::fetch::prepare::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideFetch(#[from] gix::remote::fetch::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRevwalk(#[from] gix::revision::walk::iter::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideDiffOptionsInit(#[from] gix::diff::options::init::Error),
  }

  impl<T> From<T> for super::Error
  where
    GitoxideError: From<T>,
  {
    fn from(value: T) -> Self {
      Self::Gitoxide(Box::new(value.into()))
    }
  }

}

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Repo error
pub enum Error {
  #[error("Could not find the active branch (HEAD)")]
  NoActiveBranch,
  #[error("Could not find the parent directory for the .git directory")]
  GitParent,
  #[error("Git error: {0}")]
  Git2(#[from] git2::Error),
  #[error("Gitoxide error: {0}")]
  Gitoxide(Box<errors::GitoxideError>),
  #[error("No remote named {0}")]
  MissingRemote(String),
  #[error("Missing head in remote")]
  MissingHead,
  #[error("Invalid utf8")]
  NonUtf8,
}
pub fn current_branch(repo : gix::Repository) -> anyhow::Result<String> {
  let reference =  repo
    .head_name()?
    .ok_or(Error::NoActiveBranch)?
    .to_string();
  let branch_name = reference
    .split('/')
    .last()
    .map(String::from)
    .ok_or(Error::NoActiveBranch)?;

  Ok(branch_name)
}


pub async  fn remote_latest_scoop_commit( ) -> anyhow::Result<ObjectId> {
  let  scoop_path = get_local_scoop_git()?;
  let repo = gix::open(scoop_path)?;
  let remote = repo.find_remote("origin").ok()
    .ok_or(Error::MissingRemote("origin".to_string()))?;

  
  // let config_proxy = get_config_value_no_print("proxy");
  // if !config_proxy.is_empty() {
  //   let proxy_url = if config_proxy.contains("http://") || config_proxy.contains("https://") {
  //     config_proxy.clone()
  //   } else {
  //     "http://".to_string() + &config_proxy
  //   };
  //   log::info!("proxy_option {:?}", proxy_url);
  // }

  let connection = remote.connect(gix::remote::Direction::Fetch)?;
  let (refs, _) = connection.ref_map(gix::progress::Discard, ref_map::Options::default())?;
  let remote_refs = refs.remote_refs;

  let current_branch = current_branch(repo)?;
  let head = remote_refs
    .iter()
    .find_map(|head| {
      let (name, oid, peeled) = head.unpack();
      if name == format!("refs/heads/{current_branch}") {
        if let Some(peeled) = peeled {
          Some(peeled)
        } else if let Some(oid) = oid {
          Some(oid)
        } else {
          None
        }
      } else {
        None
      }
    })
    .ok_or(Error::MissingHead)?;

  Ok(head.to_owned())
}


#[derive(Deserialize)]
struct Release {
  tag_name: String,
}

pub async fn get_latest_release_version() -> anyhow::Result<String> {
  use reqwest::header;
  let mut headers = header::HeaderMap::new();
  headers.insert("User-Agent", header::HeaderValue::from_static("hyperscoop/1.0"));
  headers.insert("Accept", header::HeaderValue::from_static("application/json"));
  let release_url = "https://api.github.com/repos/ScoopInstaller/Scoop/releases/latest";
  let client = reqwest::Client::new();
  let response = client.get(release_url).headers(headers).send().await?;
  let text = response.text().await?;
  let release = serde_json::from_str::<Release>(&text)?;
  Ok(release.tag_name)
}

pub fn  local_scoop_latest_commit( ) -> anyhow::Result<ObjectId> {
  let  scoop_path = get_local_scoop_git()?;
  let repo = gix::open(scoop_path)?;
  let  local_latest_commit = repo.head_commit()?.id();
  Ok(ObjectId::from(local_latest_commit))
}


pub fn get_local_scoop_git() -> anyhow::Result<String > {
  let scoop_path = std::env::var("SCOOP").unwrap_or_else(|_| -> String {
    let scoop = std::env::var("USERPROFILE");
    let scoop_path = scoop.unwrap_or("".into()) + "\\scoop";
    if Path::new(&scoop_path).exists() {
      return scoop_path;
    }
    return  "".to_owned() ;
  }
  );
  if  scoop_path.is_empty() {
    bail!("Scoop not found,maybe you have not installed scoop yet.");
  }
  let scoop_path = Path::new(&scoop_path).join("apps").join("scoop").join("current");
  if !scoop_path.exists() {
    bail!("Scoop not found,maybe you have not installed scoop yet.");
  }

   Ok( scoop_path .to_str().unwrap().to_owned())
}



pub   fn git_pull_update_repo_with_scoop(callback: impl Fn(Progress, bool) -> bool + Sized) -> anyhow::Result<()> {
  let scoop_path = get_local_scoop_git()?;
  let repo = git2::Repository::open(&scoop_path)?;
  let    remote = repo.find_remote("origin")?  ;
  let mut fetch_options = FetchOptions::new();
  let mut callbacks = RemoteCallbacks::new();
  callbacks.transfer_progress(|status|   callback(status, false)) ;
  fetch_options.remote_callbacks(callbacks);

  start_fetch(remote.clone()  ,&repo ,fetch_options)?;
  let stats = remote.stats()  ;
  callback(stats, true);
  Ok(())
}

pub fn git_pull_update_repo<'a> (repo_path: &str,
      callback: crate::utils::pull::ProgressCallback<'_>)
  -> anyhow::Result<()> {
  let repo = git2::Repository::open(repo_path)?;
  use   crate::utils::pull::RepoArgs;
  let remote_name =  repo.remotes()?.iter().next().unwrap_or("origin".into()).unwrap().to_string();
  // println!("remote_name:{}",remote_name);
  let remote_branch = repo.head()?.shorthand().unwrap().to_string();
  // println!("remote_branch:{}",remote_branch);

  let args = RepoArgs {
    arg_remote: Some(remote_name),
    arg_branch: Some(remote_branch),
  };
  run_pull(args,repo_path.into()  , callback)?;

  // let    remote = repo.find_remote("origin")?  ;
  // let mut fetch_options = FetchOptions::new();
  // let mut callbacks = RemoteCallbacks::new();
  // callbacks.transfer_progress(|status|   callback(status, true   )) ;
  // fetch_options.remote_callbacks(callbacks);
  //
  // start_fetch(remote.clone()  ,&repo ,fetch_options)?;
  // let stats = remote.stats()  ;
  // callback(stats, true);
  Ok(())
}
fn start_fetch(mut remote: Remote, repo: &Repository,
               mut fetch_options: FetchOptions) -> anyhow::Result<()>{
  // 执行 fetch 操作，获取远程仓库的最新数据
  remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_options), None)?;

  // 获取当前分支的引用
  let head = repo.head()?;
  let branch_name = head.shorthand().unwrap_or("master");
  let branch = repo.find_branch(branch_name, BranchType::Local)?;

  // 获取远程分支的引用
  let upstream = branch.upstream()?;
  let upstream_commit = repo.find_annotated_commit(upstream.get().peel_to_commit()?.id())?;

  // 合并远程分支到当前分支
  let analysis = repo.merge_analysis(&[&upstream_commit])?;
  if analysis.0.is_up_to_date() {
    println!("Already up to date.");
  } else if analysis.0.is_fast_forward() {
    // 快进合并
    let mut reference = head;
    reference.set_target(upstream_commit.id(), "Fast-forward")?;
    repo.set_head(reference.name().unwrap())?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    println!("Fast-forward merge completed.");
  } else {
    let head_commit = repo.find_commit(head.target().unwrap())?; // 将 Oid 转换为 Commit
    let upstream_commit = repo.find_commit(upstream_commit.id())?; // 将 Oid 转换为 Commit
    let mut index = repo.merge_commits(&head_commit, &upstream_commit, None)?;
    if index.has_conflicts() {
      println!("Merge conflicts detected. Resolve them and commit the changes.");
      return Ok(());
    }
    let tree_oid = index.write_tree_to(&repo)?;
    let tree = repo.find_tree(tree_oid)?;
    repo.commit(
      Some("HEAD"),
      &head_commit.author(),
      &head_commit.committer(),
      "Merge commit",
      &tree,
      &[&head_commit, &upstream_commit],
    )?;
    println!("Merge completed.");
  }

  Ok(())
}



mod  tests {
  #[test]
  fn test_git_pull () {
    use   crate::utils::pull::RepoArgs;
    let remote_name  = "origin" ;
    let remote_branch = "master" ;
    let args = RepoArgs {
      arg_remote: Some(remote_name.into()),
      arg_branch: Some(remote_branch.into()),
    } ;
  }
  #[test]
  fn git_pull_update() -> anyhow::Result<()> {
    use crate::utils::pull::{run };
    let repo_path :String = "A:\\scoop\\buckets\\hp".into() ;
    let repo = git2::Repository::open(&repo_path)?;
    use   crate::utils::pull::RepoArgs;

    let remote_name =  repo.remotes()?.iter().next().unwrap_or("origin".into()).unwrap().to_string();
    println!("remote_name:{}",remote_name);
    let remote_branch = repo.head()?.shorthand().unwrap().to_string();
    println!("remote_branch:{}",remote_branch);
    let args = RepoArgs {
      arg_remote: Some(remote_name),
      arg_branch: Some(remote_branch),
    };
    run(args, repo_path) ?;
    Ok(())
  }

  #[test]

  fn test_default_branch() -> anyhow::Result<()> {

    let  repo  :String = "A:\\scoop\\buckets\\cmontage".into(); 
    let  repo = git2::Repository::open(&repo)?;
    let  default_branch = repo.head()?.shorthand().unwrap_or("").to_string();
    assert_eq!(default_branch,"main");
    Ok(())
  }
}
