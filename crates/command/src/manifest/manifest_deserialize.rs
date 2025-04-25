use serde::{Deserialize, Serialize};

pub type ManifestObj = serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)] // 允许处理多种类型
pub enum StringArrayOrString {
    StringArray(Vec<String>), // 数组类型
    #[default]
    Null,
    String(String), // 字符串类型
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PSModuleStruct {
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallerUninstallerStruct {
    pub file: Option<String>,
    pub script: Option<StringArrayOrString>,
    pub args: Option<StringArrayOrString>,
    pub keep: Option<bool>,
}

#[must_use]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
// #[serde(untagged)]    // 只能用于枚举
pub struct ArchitectureObject {
    #[serde(rename = "64bit")]
    pub x64bit: Option<BaseArchitecture>,
    #[serde(rename = "32bit")]
    pub x86bit: Option<BaseArchitecture>,
    pub arm64: Option<BaseArchitecture>,
}
#[must_use]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseArchitecture {
    pub bin: Option<StringOrArrayOrDoubleDimensionArray>,
    pub checkver: Option<ObjectOrString>,
    pub extract_dir: Option<StringArrayOrString>,
    pub extract_to: Option<StringArrayOrString>,
    pub hash: Option<StringArrayOrString>,
    pub installer: Option<InstallerUninstallerStruct>,
    pub uninstaller: Option<InstallerUninstallerStruct>,
    pub url: Option<StringArrayOrString>,
    pub shortcuts: Option<ArrayOrDoubleDimensionArray>,
    pub pre_install: Option<StringArrayOrString>,
    pub post_install: Option<StringArrayOrString>,
}

impl ArchitectureObject {
    pub fn get_specific_architecture(&self, arch: &str) -> Option<&BaseArchitecture> {
        match arch {
            "64bit" => self.x64bit.as_ref(),
            "32bit" => self.x86bit.as_ref(),
            "arm64" => self.arm64.as_ref(),
            _ => None,
        }
    }
}

#[must_use]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoUpdateStruct {
    pub bin: Option<StringOrArrayOrDoubleDimensionArray>,
    pub extract_dir: Option<String>,
    pub extract_to: Option<StringArrayOrString>,
    pub note: Option<StringArrayOrString>, //  !complete

    pub hash: Option<ObjectArrayOrStringOrObjectOrStringArray>,
    pub installer: Option<ManifestObj>,
    pub uninstaller: Option<ManifestObj>,
    pub url: Option<ObjectArrayOrStringOrObjectOrStringArray>,
    pub shortcuts: Option<ArrayOrDoubleDimensionArray>,
    pub pre_install: Option<StringArrayOrString>,
    pub post_install: Option<StringArrayOrString>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectOrArray {
    #[default]
    Null,
    ManifestObj(serde_json::Value), // 对象类型
    StringArray(Vec<String>),       // 数组类型
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectOrString {
    #[default]
    Null,
    ManifestObj(serde_json::Value), // 对象类型
    String(String),                 // 字符串类型
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum ArrayOrDoubleDimensionArray {
    #[default]
    Null,
    StringArray(Vec<String>),
    DoubleDimensionArray(Vec<Vec<String>>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOrDoubleDimensionArray {
    #[default]
    Null,
    String(String), // 字符串类型
    StringArray(Vec<String>),
    DoubleDimensionArray(Vec<Vec<String>>),
    NestedStringArray(Vec<StringOrArrayOrDoubleDimensionArray>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ArrayOrStringOrObject {
    #[default]
    Null,
    String(String), // 字符串类型
    StringArray(Vec<String>),
    ManifestObj(serde_json::Value),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectArrayOrStringOrObject {
    #[default]
    Null,
    String(String), // 字符串类型
    ObjectArray(Vec<ManifestObj>),
    ManifestObj(serde_json::Value),
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectArrayOrStringOrObjectOrStringArray {
    #[default]
    Null,
    String(String), // 字符串类型
    ObjectArray(Vec<ManifestObj>),
    ManifestObj(serde_json::Value),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOrDotDimensionArrayOrObject {
    #[default]
    Null,
    ManifestObj(serde_json::Value),
    String(String), // 字符串类型
    StringArray(Vec<String>),
    DoubleDimensionArray(Vec<Vec<String>>),
}

mod tests {
    #[test]
    fn test_serde_json() {
        let json = r#"
       {
         "name": "hp",
         "version": "1.0.0",
         "architecture": {
           "amd64": {
             "installer": "innosetup"
           }
         },
           "shortcuts": [ [ "zigmod.exe" , "zigmod fuck you" ] ],
         "suggest" : {
         "JDK": "1.8+"
         },
         "depends" : {
         "JDK":   ["8", "9","22" ,"GraalVM"]
         }
       }
     "#;
        let manifest: serde_json::Value = serde_json::from_str(json).unwrap();
        let suggest = manifest["suggest"].as_object().unwrap();
        let depends = manifest["depends"].as_object().unwrap();
        let shortcuts = manifest["shortcuts"].as_array().unwrap();
        println!("suggest {:?}", suggest);
        println!("depends {:?}", depends.values().collect::<Vec<_>>());
        println!("shortcuts {:?}", shortcuts);
    }
    #[test]
    fn test_manifest_file() {
        use crate::manifest::install_manifest::InstallManifest;
        let file = r"A:\Scoop\buckets\ScoopMaster\bucket\zigmod.json";
        let content = std::fs::read_to_string(file).unwrap();
        let manifest: InstallManifest = serde_json::from_str(&content).unwrap();
        let shortcuts = manifest.shortcuts;
        if shortcuts.is_some() {
            let shortcuts = shortcuts.unwrap();
            println!("shortcuts {:?}", shortcuts);
        } else {
            println!("shortcuts is none ");
        }
    }
}
