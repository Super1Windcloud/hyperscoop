#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InstallOptions {
    NoUseDownloadCache,
    NoAutoDownloadDepends,
    SkipDownloadHashCheck,
    ArchOptions(String),
    UpdateHpAndBuckets,
    OnlyDownloadNoInstall,
    Global,
} 



#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum  UpdateOptions {
    NoUseDownloadCache,
    NoAutoDownloadDepends,
    SkipDownloadHashCheck,
    UpdateHpAndBuckets,
    Global,
    UpdateAllAPP, 
    RemoveOldVersionApp 
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ArchiveFormat {
    SevenZip,
    ZIP,
    GZIP,
    XZIP,
    BZIP2,
    ZSTD,
    RAR,
    EXE,
    MSI,
    TAR,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HashFormat {
    MD5,
    SHA1,
    SHA256,
    SHA512,
}
