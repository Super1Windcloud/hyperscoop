


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InstallOptions {
    NoUseDownloadCache,
    NoAutoDownloadDepends,
    SkipDownloadHashCheck,
    ArchOptions,
    UpdateHpAndBuckets,
}
