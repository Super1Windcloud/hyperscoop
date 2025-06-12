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
    InstallSpecialVersionApp,
    InstallSpecialBucketApp, // 单元变体（无数据）
    CurrentInstallApp {
        app_name: String,
        app_version: String,
    }, // 结构体变体
    AppName(Option<String>), //元组变体
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
    Shell,
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
