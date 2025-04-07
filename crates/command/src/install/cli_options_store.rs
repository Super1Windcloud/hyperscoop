


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InstallOptions {
    NoUseDownloadCache,
    NoAutoDownloadDepends,
    SkipDownloadHashCheck,
    ArchOptions(String),
    UpdateHpAndBuckets,
  OnlyDownloadNoInstall 
}







#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ArchiveFormat  {
     SevenZip, 
     ZIP, 
     RAR , 

}




