use crate::manifest::manifest_deserialize::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[derive(Default)]
pub enum SuggestObjValue {
    #[default]
    Null,
    String(String),
    StringArray(Vec<String>),
}

pub type SuggestObj =
    std::collections::HashMap<String, crate::manifest::install_manifest::SuggestObjValue>;

#[must_use]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateManifest {
    #[serde(skip)]
    pub(crate) name: Option<String>,

    pub checkver: Option<CheckverStruct>,

    pub homepage: Option<String>,

    pub version: Option<String>,

    pub autoupdate: Option<AutoUpdateStruct>,

    #[serde(skip)]
    bucket: Option<String>,

    #[serde(skip)]
    pub installer: Option<InstallerUninstallerStruct>,

    #[serde(skip)]
    pub shortcuts: Option<ArrayOrDoubleDimensionArray>,

    #[serde(skip)]
    pub architecture: Option<ArchitectureObject>,

    #[serde(skip)]
    pub depends: Option<String>,

    #[serde(skip)]
    pub bin: Option<StringOrArrayOrDoubleDimensionArray>,

    #[serde(skip)]
    pub url: Option<StringArrayOrString>,

    #[serde(skip)]
    pub hash: Option<StringArrayOrString>,

    #[serde(skip)]
    pub extract_dir: Option<StringArrayOrString>, // 数组中元素和URL数组一一对应

    #[serde(skip)]
    pub extract_to: Option<StringArrayOrString>, // 数组中元素和URL数组一一对应

    #[serde(skip)]
    pub env_add_path: Option<StringArrayOrString>, // !complete

    #[serde(skip)]
    pub innosetup: Option<bool>,

    #[serde(skip)]
    pub notes: Option<StringArrayOrString>, //  !complete

    #[serde(skip)]
    pub persist: Option<StringOrArrayOrDoubleDimensionArray>,

    #[serde(skip)]
    pub psmodule: Option<PSModuleStruct>,

    #[serde(skip)]
    pub env_set: Option<ManifestObj>, // !complete

    #[serde(skip)]
    pub pre_install: Option<StringArrayOrString>, // 安装前执行的命令
    #[serde(skip)]
    pub post_install: Option<StringArrayOrString>, // 安装后执行的命令

    #[serde(skip)]
    pub description: Option<String>,

    #[serde(skip)]
    pub uninstaller: Option<InstallerUninstallerStruct>,

    #[serde(skip)]
    pub pre_uninstall: Option<StringArrayOrString>,

    #[serde(skip)]
    pub post_uninstall: Option<StringArrayOrString>,

    #[serde(skip)]
    pub license: Option<ObjectOrString>,

    #[serde(rename = "##")]
    #[serde(skip)]
    manifest_comment: Option<StringArrayOrString>,
}
