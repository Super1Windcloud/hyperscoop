#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InstallOptions<'a> {
    NoUseDownloadCache,
    NoAutoDownloadDepends,
    SkipDownloadHashCheck,
    ArchOptions(&'a str),
    UpdateHpAndBuckets,
    OnlyDownloadNoInstall,
    ForceDownloadNoInstallOverrideCache,
    CheckCurrentVersionIsLatest,
    Global,
    ForceInstallOverride,
    UpdateTransaction,
    InteractiveInstall, 
    InstallSpecialVersionApp , 
    InstallSpecialBucketApp 
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UpdateOptions {
    NoUseDownloadCache,
    NoAutoDownloadDepends,
    SkipDownloadHashCheck,
    UpdateHpAndBuckets,
    Global,
    UpdateAllAPP,
    RemoveOldVersionApp,
    ForceUpdateOverride,
    InteractiveInstall,
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
    INNO,
    MSI,
    TAR,
    Other,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum HashFormat {
    MD5,
    SHA1,
    SHA256,
    SHA512,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DownloadState {
    Queued,
    Downloading { progress: f64, speed: f64 },
    Paused,
    Completed(String),
    Failed(String),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ParserUrl {
    ExternalUrl(String),
    InternalUrl(String),
}
