use std::path::Path;
use anyhow::bail;
use gix::{
  bstr::BStr, remote::ref_map, revision::walk::Sorting,
  traverse::commit::simple::CommitTimeOrder, Commit, ObjectId, Repository,
};
use serde::Deserialize;

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
