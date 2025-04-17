use crate::init_env::{
    get_all_buckets_dir_child_bucket_path, get_all_global_buckets_dir_child_bucket_path,
};
use anyhow::bail;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::list::VersionJSON;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    #[serde(skip)]
    bucket: Option<String>,
    #[serde(skip)]
    name: Option<String>,

    /// 版本号
    pub version: Option<String>,

    // 与installer相同的选项，但运行文件/脚本来卸载应用程序。
    pub uninstaller: Option<String>,

    //运行非 MSI 安装程序的说明。
    pub installer: Option<String>, // 安装程序的名称，如 `innosetup`

    //  指定在开始菜单中可用的快捷方式值
    pub shortcuts: Option<String>, // 快捷方式配置

    //如果应用程序不是 32 位，则可以使用架构来包装差异
    pub architecture: Option<String>,
    //定义如何自动更新清单。
    pub autoupdate: Option<String>,
    pub cookie: Option<HashMap<String, Option<serde_json::Value>>>,
    //将自动安装的应用程序的运行时依赖项
    pub depends: Option<String>,
    pub description: Option<String>,
    //应用程序维护人员和开发人员可以使用bin/checkver工具来检查应用程序的更新版本
    // 。清单中的checkver属性是一个正则表达式，可用于匹配应用程序主页中应用程序的当前稳定版本
    pub checkver: Option<String>, // 用于检查更新的配置

    //在用户路径上可用的程序（可执行文件或脚本）的字符串或字符串数 组
    pub bin: Option<String>,      //可执行文件所在的目录。
    pub checksum: Option<String>, //文件的校验和
    /**
    要下载的一个或多个文件的 URL。如果有多个 URL，可以使用 JSON 数组，例如 "url": [ "http://example.org/program.zip", "http://example.org/dependencies.zip" ] 。 URL 可以是 HTTP、HTTPS 或 FTP。

      To change the filename of the downloaded URL, you can append a URL fragment (starting with #) to URLs. For examples,
      要更改下载的 URL 的文件名，您可以将 URL 片段（以#开头）附加到 URL。例如，
      "http://example.org/program.exe" -> "http://example.org/program.exe#/dl.7z"
      Note the fragment must start with #/ for this to work.
      请注意，片段必须以#/开头才能正常工作。
      In the above examples, Scoop will download program.exe but save it as dl.7z, which will then be extracted automatically with 7-Zip. This technique is commonly used in Scoop manifests to bypass executable installers which might have undesirable side-effects like registry changes, files placed outside the install directory, or an admin elevation prompt.
      在上面的示例中，Scoop 将下载program.exe ，但将其另存为dl.7z ，然后使用 7-Zip 自动解压。此技术通常在 Scoop 清单中使用，以绕过可执行安装程序，这些安装程序可能会产生不良副作用，例如注册表更改、放置在安装目录之外的文件或管理员提升提示。
    */
    pub url: Option<String>,

    //字符串或字符串数组，其中包含url中每个 URL 的文件哈希值。默认情况下，
    // 哈希值是 SHA256，但您可以通过在哈希字符串前添加“sha512:”、“sha1:”或“md5:”前缀来使用 SHA512、SHA1 或 MD5
    pub hash: Option<String>,
    //   如果url指向压缩文件（支持 .zip、.7z、.tar、.gz、.lzma 和 .lzh），Scoop 将仅提取其中指定的目录
    pub extract_dir: Option<String>,
    // 如果url指向压缩文件（支持 .zip、.7z、.tar、.gz、.lzma 和 .lzh），Scoop 会将所有内容提取到指定目录
    pub extract_to: Option<String>, // 解压缩后的目录
    ///  主页 URL
    pub homepage: Option<String>,
    //将此目录添加到用户路径（如果使用--global则添加到系统路径）。
    // 该目录是相对于安装目录的，并且必须位于安装目录内。
    pub env_add_path: Option<String>, // 添加到 PATH 环境变量的路径。
    //如果安装程序基于 InnoSetup，则设置为布尔值true
    pub innosetup: Option<bool>,
    pub license: Option<String>,

    //：单行字符串或字符串数组，其中包含在安装应用程序后显示的消息。
    pub notes: Option<String>,

    //保存在应用程序的数据目录中的目录和文件的字符串或字符串数组。持久数据
    pub persist: Option<String>,

    //作为 PowerShell 模块安装在~/scoop/modules中。
    pub psmodule: Option<String>,

    /// 为用户（或系统，如果使用--global ）设置一个或多个环境变量（
    pub env_set: Option<String>,

    pub pre_install: Option<String>,    // 安装前执行的命令
    pub post_install: Option<String>,   // 安装后执行的命令
    pub pre_uninstall: Option<String>,  // 卸载前执行的命令
    pub post_uninstall: Option<String>, // 卸载后执行的命令
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MainBucket {
    Main,
    Extras,
    Versions,
    Other,
}

pub fn get_latest_app_version_from_local_bucket(app_name: &str) -> anyhow::Result<String> {
     let  better_manifest = get_latest_manifest_from_local_bucket(app_name)?; 
     if !Path::new(&better_manifest).exists() { 
       bail!("Manifest {}does not exist", better_manifest.display());
     } 
     let content = std::fs::read_to_string(&better_manifest)?; 
     let version: VersionJSON = serde_json::from_str(&content)?;
    if version.version.is_none() {
      bail!("该App没有找到版本信息,manifest.json格式错误")
    }
    Ok(version.version.unwrap())
}
pub fn get_latest_app_version_from_local_bucket_global(app_name: &str) -> anyhow::Result<String> {
  let  better_manifest = get_latest_manifest_from_local_bucket_global(app_name)?;
  if !Path::new(&better_manifest).exists() {
    bail!("Manifest {}does not exist", better_manifest.display());
  }
  let content = std::fs::read_to_string(&better_manifest)?;
  let version: VersionJSON = serde_json::from_str(&content)?;
  if version.version.is_none() {
    bail!("该App没有找到版本信息,manifest.json格式错误")
  }
  Ok(version.version.unwrap())
}


pub fn get_all_manifest_files_from_bucket<'a>(
    all_buckets_root: &'a [String],
    app_name: &'a str,
) -> Vec<(PathBuf, MainBucket)> {
    let all_manifests_file = all_buckets_root
        .par_iter()
        .map(|bucket_dir| {
            let child_files = std::fs::read_dir(bucket_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let file_type = e.file_type().unwrap();
                    file_type.is_file()
                })
                .map(|e| e.path())
                .collect::<Vec<_>>();
            child_files
        })
        .flatten()
        .collect::<Vec<_>>();
    let manifest_path = all_manifests_file
        .par_iter()
        .filter_map(|path| {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            if file_name.to_lowercase() == app_name.to_lowercase() {
                let parent = path.parent().unwrap().parent().unwrap();
                let bucket_name = parent.file_name().unwrap().to_str().unwrap();
                let bucket_enum = match bucket_name.to_lowercase().as_str() {
                    "main" => MainBucket::Main,
                    "extras" => MainBucket::Extras,
                    "versions" => MainBucket::Versions,
                    _ => MainBucket::Other,
                };
                Some((path.as_path().to_path_buf(), bucket_enum))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    manifest_path
}
pub fn get_latest_manifest_from_local_bucket(app_name: &str) -> anyhow::Result<PathBuf> {
    let all_buckets_root = get_all_buckets_dir_child_bucket_path()?;
    let result = get_all_manifest_files_from_bucket(all_buckets_root.as_slice(), app_name);
    if result.is_empty() {
        bail!("No manifest found for '{app_name}'");
    }
    let app_manifest_path = find_better_bucket(result);

    Ok(app_manifest_path)
}
fn find_better_bucket(result: Vec<(PathBuf, MainBucket)>) -> PathBuf {
    let final_path = if result.iter().any(|(_, bucket)| *bucket == MainBucket::Main) {
        result
            .iter()
            .find(|(_, bucket)| *bucket == MainBucket::Main)
            .unwrap()
            .0
            .clone()
    } else if result
        .iter()
        .any(|(_, bucket)| *bucket == MainBucket::Extras)
    {
        result
            .iter()
            .find(|(_, bucket)| *bucket == MainBucket::Extras)
            .unwrap()
            .0
            .to_path_buf()
    } else if result
        .iter()
        .any(|(_, bucket)| *bucket == MainBucket::Versions)
    {
        result
            .iter()
            .find(|(_, bucket)| *bucket == MainBucket::Versions)
            .unwrap()
            .0
            .to_owned()
    } else {
        result.first().unwrap().0.to_owned()
    };
    final_path
}
pub fn get_latest_manifest_from_local_bucket_global(app_name: &str) -> anyhow::Result<PathBuf> {
    let all_buckets_root = get_all_global_buckets_dir_child_bucket_path()?;
    let result = get_all_manifest_files_from_bucket(all_buckets_root.as_slice(), app_name);

    if result.is_empty() {
        bail!("No manifest found for '{app_name}'");
    }
    let app_manifest_path = find_better_bucket(result);

    Ok(app_manifest_path)
}


mod test_manifest {
    #[test]
    fn test_output() {
        use super::*;
        get_latest_manifest_from_local_bucket("zigmod").unwrap();
    }
}
