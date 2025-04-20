


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum  LifecycleScripts {
    PreInstall,
    PostInstall,
    Installer, 
    Uninstaller, 
    PreUninstall,
    PostUninstall,
  
}

pub  fn  parse_lifecycle_scripts(scripts: &str) -> anyhow::Result<()> {

  
  Ok(() )
}