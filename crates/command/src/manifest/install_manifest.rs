use crate::manifest::manifest_deserialize::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[derive(Default)]
pub enum SuggestObjValue {
    #[default]
    Null,
    String(String),
    StringArray(Vec<String>),
}

pub type SuggestObj = std::collections::HashMap<String, SuggestObjValue>;

#[must_use]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallManifest {
    #[serde(skip)]
    bucket: Option<String>,
    #[serde(skip)]
    pub(crate) name: Option<String>,

    /// 版本号
    pub version: Option<String>, // !complete

    ///运行非 MSI 安装程序的说明。
    /**
     file ：安装程序可执行文件。对于installer默认为最后下载的 URL。必须为uninstaller指定。
    script ：作为安装程序/卸载程序而不是file执行的命令的单行字符串或字符串数组。
    args ：传递给安装程序的参数数组。选修的。
    keep ：如果安装程序在运行后应保留，则为"true" （例如，用于将来的卸载）。
    如果省略或设置为任何其他值，安装程序将在运行后被删除。有关示例，请参阅extras/oraclejdk
    。在uninstaller指令中使用时，此选项将被忽略。
    script和args可用的变量： $fname （上次下载的文件）， $manifest （反序列化的清单引用），
    $architecture （ 64bit或32bit ）， $dir （安装目录）
    在scoop install和scoop update期间调用。*/
    pub installer: Option<InstallerUninstallerStruct>,

    /**
    该数组必须包含可执行文件/标签对。第三和第四个元素是可选的。
     目标文件的路径[必需]
     快捷方式的名称（支持子目录： <AppsSubDir>\\<AppShortcut>例如sysinternals ）[必需]
     启动参数[可选]
     图标文件的路径[可选]
    */
    pub shortcuts: Option<ArrayOrDoubleDimensionArray>,

    /**
    如果应用程序不是 32 位，则可以使用架构来包装差异（示例）。
         "64bit": {
              "url": "https://www.7-zip.org/a/7z2409-x64.msi",
              "hash": "ec6af1ea0367d16dde6639a89a080a524cebc4d4bedfe00ed0cac4b865a918d8",
              "extract_dir": "Files\\7-Zip"
          },
          "32bit": {
              "url": "https://www.7-zip.org/a/7z2409.msi",
              "hash": "c7f182dad21eebfce02f141d6a01f847d1e194c4d6aa29998d9305388553cf6a",
              "extract_dir": "Files\\7-Zip"
          },
          "arm64": {
              "url": "https://www.7-zip.org/a/7z2409-arm64.exe",
              "hash": "bc7b3a18f218f4916e1c4996751468f96e46eb7e97e91e8c1553d74793037f1a",
              "pre_install": [
                  "$7zr = Join-Path $env:TMP '7zr.exe'",
                  "Invoke-WebRequest https://www.7-zip.org/a/7zr.exe -OutFile $7zr",
                  "Invoke-ExternalCommand $7zr @('x', \"$dir\\$fname\", \"-o$dir\", '-y') | Out-Null",
                  "Remove-Item \"$dir\\Uninstall.exe\", \"$dir\\*-arm64.exe\", $7zr"
              ]
          }
      32bit|64bit|arm64 ：包含特定于体系结构的指令（ bin 、 checkver 、 extract_dir 、 hash 、 installer 、
    pre_install 、 post_install 、 shortcuts 、 uninstaller 、 url  */
    pub architecture: Option<ArchitectureObject>,
    ///定义如何自动更新清单。

    ///将自动安装的应用程序的运行时依赖项。另请参阅suggest （如下）
    pub depends: Option<String>,

    /**
    显示一条消息，建议提供补充功能的可选应用程序。请参阅ant的示例。
      ["Feature Name"] = [ "app1", "app2"... ]
      例如 "JDK": [ "extras/oraclejdk", "openjdk" ]
      如果已经安装了针对该功能建议的任何应用程序，则该功能将被视为“已完成”，并且用户将不会看到任何建议。
    */
    pub suggest: Option<SuggestObj>, // !complete
    /**
    在用户路径上可用的程序（可执行文件或脚本）的字符串或字符串数组。
     您还可以创建一个别名填充程序，它使用与实际可执行文件不同的名称，
    并（可选）将参数传递给可执行文件。不要仅使用可执行文件的字符串，而是使用例如：
    [ "program.exe", "alias", "--arguments" ] 。请参阅busybox 的示例。
     但是，如果您仅声明这样的垫片，则必须确保它包含在外部阵列中，
    例如： "bin": [ [ "program.exe", "alias" ] ] 。否则，它将被读为单独的垫片。
    */
    pub bin: Option<StringOrArrayOrDoubleDimensionArray>,

    /**
    要下载的一个或多个文件的 URL。如果有多个 URL，可以使用 JSON 数组，例如 "url":
    [ "http://example.org/program.zip", "http://example.org/dependencies.zip" ] 。
      URL 可以是 HTTP、HTTPS 或 FTP。
      要更改下载的 URL 的文件名，您可以将 URL 片段（以#开头）附加到 URL。例如，
      "http://example.org/program.exe" -> "http://example.org/program.exe#/dl.7z"
      请注意，片段必须以#/开头才能正常工作。
      在上面的示例中，Scoop 将下载program.exe ，但将其另存为dl.7z ，然后使用 7-Zip 自动解压。
    此技术通常在 Scoop 清单中使用，以绕过可执行安装程序，这些安装程序可能会产生不良副作用，
    例如注册表更改、放置在安装目录之外的文件或管理员提升提示。
    */
    pub url: Option<StringArrayOrString>,

    ///字符串或字符串数组，其中包含url中每个 URL 的文件哈希值。默认情况下，
    /// 哈希值是 SHA256，但您可以通过在哈希字符串前添加“sha512:”、“sha1:”或“md5:”前缀来使用 SHA512、SHA1 或 MD5
    pub hash: Option<StringArrayOrString>,
    ///   如果url指向压缩文件（支持 .zip、.7z、.tar、.gz、.lzma 和 .lzh），Scoop 将仅提取其中指定的目录
    pub extract_dir: Option<StringArrayOrString>, // 数组中元素和URL数组一一对应
    /// 如果url指向压缩文件（支持 .zip、.7z、.tar、.gz、.lzma 和 .lzh），Scoop 会将所有内容提取到指定目录
    pub extract_to: Option<StringArrayOrString>, // 数组中元素和URL数组一一对应
    ///将此目录添加到用户路径（如果使用--global则添加到系统路径）。
    /// 该目录是相对于安装目录的，并且必须位于安装目录内。
    pub env_add_path: Option<StringArrayOrString>, // !complete
    ///如果安装程序基于 InnoSetup，则设置为布尔值true
    pub innosetup: Option<bool>,

    ///：单行字符串或字符串数组，其中包含在安装应用程序后显示的消息。
    pub notes: Option<StringArrayOrString>, //  !complete

    ///保存在应用程序的数据目录中的目录和文件的字符串或字符串数组。持久数据 ,
    ///  如果是二维 NestedStringArray(Vec<StringOrArrayOrDoubleDimensionArray>),
    /// 那么 其中的String依旧是要建立符号链接的子目录, 而Array里面的只能有两个元素,
    ///  "persist": [
    ///         "cli",
    ///         [
    ///              "php.ini-production",
    /// /            "cli\\php.ini"
    ///         ]
    ///     ],
    /// 第一个元素是源目录的文件或者目录  , 第二个元素是要复制到Persist的目标路径
    /// 如果是持久化文件, 直接指定复制的文件目标路径,可以对文件重命名,如果指定父目录的话,文件名不变
    /// 如果是持久化目录 , 直接指定目标路径, 可以对目录重命名,  如果指定父目录的话,目录名不变
    pub persist: Option<StringOrArrayOrDoubleDimensionArray>,

    ///作为 PowerShell 模块安装在~/scoop/modules中。
    ///name （ psmodule必需）：模块的名称，该名称应与解压目录中的至少一个文件匹配，以便 PowerShell 将其识别为模块。
    pub psmodule: Option<PSModuleStruct>,

    /// 为用户（或系统，如果使用--global ）设置一个或多个环境变量
    pub env_set: Option<ManifestObj>, // !complete
    /**
    appdir
    参考另一个勺应用程序。例如，要检查是否安装了另一个应用程序，您可以使用：
    "post_install": [ "if (Test-Path \"$(appdir otherapp)\\current\\otherapp.exe\") { <# .. do something .. #> }"
    $scoopdir
    基本scoop安装dir（通常为%USERPROFILE%\scoop ，``scoop％覆盖）
    $dir
    对于pre_install ， pre_uninstall ， post_uninstall ， installer.script和uninstaller.script字段是 app_path/version
    对于 post_install 是 app_path/current
    $oldscoopdir ,  C:\Users\username\AppData\Local\scoop
    $original_dir  : C:\Users\username\scoop\apps\$app\1.2.3
    $modulesdir ,   %USERPROFILE%\scoop\modules
    $globaldir ,   C:\ProgramData\scoop ,通常%ProgramData\scoop ， %SCOOP_GLOBAL%覆盖）
    $cfgpath  ,   ~/.scoop
    $cachedir , %USERPROFILE%\scoop\cache
    $bucketsdir , %USERPROFILE%\scoop\buckets
    $dir ,  %USERPROFILE%\scoop\apps\$app\$version
    $persist_dir,  %USERPROFILE%\scoop\persist\$app
    $version , manifest Version
    $manifest  , manifest Object
    $global ,   $false or $true
    $cfg,      scoop config Object, powershell object  ,{SCOOP_BRANCH, SCOOP_REPO, lastupdate, etc}
    $cmd ,  uninstall, update, install ,目前正在运行的子命令
    $architecture ,  64bit或32bit  已安装应用程序的CPU架构
    $app  ,  应用程序的名称（清单文件的名称） ,
    */
    pub pre_install: Option<StringArrayOrString>, // 安装前执行的命令
    /// 安装应用程序后要执行的命令的一行字符串或字符串数组。这些可以使用$dir 、 $persist_dir和$version等变量
    pub post_install: Option<StringArrayOrString>, // 安装后执行的命令

    #[serde(skip)]
    pub description: Option<String>,

    /// bucket维护人员用于自动更新当前 manifest的 Version和 URL和 Hash值
    #[serde(skip)]
    pub autoupdate: Option<AutoUpdateStruct>,
    #[serde(skip)]
    pub homepage: Option<String>,

    ///  与installer相同的选项，但运行文件/脚本来卸载应用程序。在scoop uninstall和scoop update期间调用
    pub uninstaller: Option<InstallerUninstallerStruct>,
    pub pre_uninstall: Option<StringArrayOrString>,  // 卸载前执行的命令
    pub post_uninstall: Option<StringArrayOrString>, // 卸载后执行的命令
    /**
    序的软件许可证的字符串或哈希值。对于知名许可证，请使用https://spdx.org/licenses中找到的标识符。
    对于其他许可证，请使用许可证的 URL（如果有）。否则，请酌情使用“免费软件”、“专有软件”、“公共领域”、
    “共享软件”或“未知”（定义如下）。如果不同的文件有不同的许可证，请用逗号（,）分隔许可证。
    如果整个应用程序是双重许可的，则使用管道符号 (|) 分隔许可证。
      identifier ：SPDX 标识符，或“免费软件”（永久免费使用）、“专有”（必须付费使用）、“公共领域”、
    “共享软件”（免费试用，最终必须付费）或“未知”（无法确定许可证），视情况而定。
      url ：对于非 SPDX 许可证，请包含许可证的链接。也可以包含 SPDX 许可证的链接。
    */
    #[serde(skip)]
    pub license: Option<ObjectOrString>,
    /// 包含注释的单行字符串或字符串数组
    #[serde(rename = "##")]
    #[serde(skip)]
    manifest_comment: Option<StringArrayOrString>,

    ///应用程序维护人员和开发人员可以使用bin/checkver工具来检查应用程序的更新版本
    /// 。清单中的checkver属性是一个正则表达式，可用于匹配应用程序主页中应用程序的当前稳定版本
    #[serde(skip)]
    pub checkver: Option<ObjectOrString>, // 用于检查更新的配置
}

impl InstallManifest {
    pub fn set_name(&mut self, path: &str) -> &mut Self {
        let app_name = Path::new(path).file_stem().unwrap().to_str().unwrap();
        self.name = Some(app_name.to_string());
        self
    }
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
}

#[cfg(test)]
mod test_manifest_deserialize {
    #[allow(unused_imports)]
    use super::*;
    use crate::install::show_suggest;
    #[allow(unused_imports)]
    use rayon::prelude::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_install_manifest() {
        use crate::buckets::get_buckets_path;
        use std::path::Path;
        use std::sync::{Arc, Mutex};
        let bucket = get_buckets_path().unwrap();
        let buckets = bucket
            .iter()
            .par_bridge()
            .map(|path| Path::new(path).join("bucket"))
            .collect::<Vec<_>>();
        let files = buckets
            .iter()
            .flat_map(|path| path.read_dir().unwrap().map(|res| res.unwrap().path()))
            .collect::<Vec<_>>();
        let _count = Arc::new(Mutex::new(0));
        for path in files {
            let content = std::fs::read_to_string(&path);
            if content.is_err() {
                println!("decode   error {:?}", path.display());
                continue;
            }
            let content = content.unwrap();
            let manifest = serde_json::from_str::<InstallManifest>(&content);
            if manifest.is_err() {
                continue;
            }

            let _manifest: InstallManifest = manifest.unwrap();
            // if find_extract(_manifest, &path, &_count) {
            //     return;
            // };
            //  if  find_url_and_hash(_manifest, &path, &_count ) { return; };
            // if  find_suggest_and_depends(_manifest, path , &_count ) { return; };
            //  find_architecture_test(_manifest, path);
            // if find_env_set(_manifest, path, &_count) {
            //     return;
            // }
            /*    if find_bin_and_shortcuts(_manifest, &path, &_count) {
                return;
            };*/
            // if  find_persist(_manifest, &path, &_count ) { return; };
            // if find_install_and_uninstall(_manifest, path, &_count) {
            //     return;
            // };

            if find_innosetup_exe(_manifest, path, &_count) {
                return;
            };
        }
        pub fn find_innosetup_exe(
            _manifest: InstallManifest,
            path: PathBuf,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let innosetup = _manifest.innosetup;
            if innosetup.is_some() {
                println!("innosetup is {}", innosetup.unwrap());
                println!("path  {}", path.display());
                *_count.lock().unwrap() += 1;
                if *_count.lock().unwrap() >= 2 {
                    return true;
                }
            }
            false
        }
        pub fn find_install_and_uninstall(
            _manifest: InstallManifest,
            path: PathBuf,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let installer = _manifest.installer;
            let uninstaller = _manifest.uninstaller;
            let architecture = _manifest.architecture;
            if architecture.is_some() {
                let architecture = architecture.unwrap();
                let x64 = architecture.x64bit;
                if x64.is_some() {
                    let x64 = x64.unwrap();
                    let installer = x64.installer;
                    let uninstaller = x64.uninstaller;
                    if installer.is_some() {
                        let installer = installer.unwrap();
                        println!("installer  {:?}", installer);
                        println!("path  {}", path.display());
                        // *_count.lock().unwrap() += 1;
                    }
                    if uninstaller.is_some() {
                        let uninstaller = uninstaller.unwrap();
                        println!("uninstaller  {:?}", uninstaller);
                        println!("path  {}", path.display());
                        // *_count.lock().unwrap() += 1;
                    }
                }
            }

            if installer.is_some() {
                let installer = installer.unwrap();
                println!("installer  {:?}", installer);
                println!("path  {}", path.display());
                *_count.lock().unwrap() += 1;
                if *_count.lock().unwrap() >= 10 {
                    return true;
                }
            }
            if uninstaller.is_some() {
                let uninstaller = uninstaller.unwrap();
                println!("uninstaller  {:?}", uninstaller);
                println!("path  {}", path.display());
                *_count.lock().unwrap() += 1;
                if *_count.lock().unwrap() >= 10 {
                    return true;
                }
            }
            false
        }
        fn find_persist(
            _manifest: InstallManifest,
            path: &PathBuf,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let persist = _manifest.persist;
            if persist.is_some() {
                let persist = persist.unwrap();
                match persist {
                    StringOrArrayOrDoubleDimensionArray::StringArray(array) => {
                        // println!("persist {:?}", array);
                        // println!(" path {}", path.display());
                        // *_count.lock().unwrap() += 1;
                    }
                    StringOrArrayOrDoubleDimensionArray::String(str) => {
                        // println!("persist {:?}", str);
                    }
                    StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(double_arr) => {
                        println!("persist {:?}", &double_arr);
                        println!(" path {}", path.display());
                        *_count.lock().unwrap() += 1;
                    }
                    StringOrArrayOrDoubleDimensionArray::Null => {
                        println!("persist {:?}", "Null");
                        println!(" path {}", path.display());
                    }
                    StringOrArrayOrDoubleDimensionArray::NestedStringArray(_) => {}
                }
                if *_count.lock().unwrap() >= 3 {
                    return true;
                }
            }

            false
        }
        fn find_bin_and_shortcuts(
            _manifest: InstallManifest,
            path: &PathBuf,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let bin = _manifest.bin;
            let shortcuts = _manifest.shortcuts;
            if shortcuts.is_some() {
                let shortcuts = shortcuts.unwrap();
                let result = match shortcuts {
                    ArrayOrDoubleDimensionArray::StringArray(array) => {
                        println!("shortcuts {:?}", array);
                        println!(" path {}", path.display());
                        *_count.lock().unwrap() += 1;
                        array
                    }
                    ArrayOrDoubleDimensionArray::DoubleDimensionArray(double_arr) => {
                        // println!("shortcuts {:?}", &double_arr);
                        // println!(" path {}", path.display());
                        double_arr.into_iter().flatten().collect::<Vec<String>>()
                    }
                    ArrayOrDoubleDimensionArray::Null => {
                        println!("shortcuts {:?}", "Null");
                        println!(" path {}", path.display());
                        vec![]
                    }
                };

                if *_count.lock().unwrap() >= 2 {
                    return true;
                }
            }
            false
        }
        fn find_extract(
            _manifest: InstallManifest,
            path: &PathBuf,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let _extract_dir = _manifest.extract_dir;
            let _extract_to = _manifest.extract_to;
            if _extract_dir.is_some() {
                let dir = _extract_dir.unwrap();
                let _array = if let StringArrayOrString::StringArray(array) = dir {
                    *_count.lock().unwrap() += 1;
                    println!("{:?}", array);
                    println!(" path {}", path.display());
                    array
                } else if let StringArrayOrString::String(str) = dir {
                    vec![str]
                } else {
                    vec![]
                };
                if _count.lock().unwrap().to_owned() >= 3 {
                    return true;
                }
            }
            false
        }

        fn find_url_and_hash(
            manifest: InstallManifest,
            path: &Path,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let url = manifest.url;
            let hash = manifest.hash;
            if url.is_some() && hash.is_some() {
                let hash = hash.unwrap();
                let hash_arr = match hash {
                    StringArrayOrString::StringArray(array) => array,
                    StringArrayOrString::String(hash) => {
                        vec![hash]
                    }
                    StringArrayOrString::Null => {
                        vec![]
                    }
                };
                let result = hash_arr
                    .iter()
                    .filter(|hash| hash.contains("sha"))
                    .collect::<Vec<&String>>();
                if result.len() > 1 {
                    println!("hash {:?}", result);
                    println!("url {:?}", url.unwrap());
                    println!(" path {}", path.display());
                    *_count.lock().unwrap() += 1;
                }
                if *_count.lock().unwrap() >= 2 {
                    return true;
                }
            }
            let architecture = manifest.architecture;
            if architecture.is_some() {}
            false
        }
        fn find_suggest_and_depends(
            manifest: InstallManifest,
            path: PathBuf,
            _count: &Arc<Mutex<i32>>,
        ) -> bool {
            let suggestion = manifest.suggest;
            if suggestion.is_some() {
                let suggestion = suggestion.unwrap();
                show_suggest(&suggestion).unwrap();
                println!(" path {}", path.display());
                *_count.lock().unwrap() += 1;
                if *_count.lock().unwrap() >= 10 {
                    return true;
                }
            }
            let depends = manifest.depends;
            if depends.is_some() {
                // let depends = depends.unwrap();
                // println!("depends {:?}", depends);
                // println!(" path {}", path.display());
            }
            false
        }
    }

    #[allow(unused)]
    #[ignore]
    fn find_architecture_test(manifest: InstallManifest, path: PathBuf) -> bool {
        let architecture = manifest.clone().architecture;
        if architecture.is_some() {
            let architecture = architecture.unwrap();
            let x64 = architecture.x64bit;
            if x64.is_some() {
                let x64 = x64.unwrap();
                let installer = x64.installer;
                if installer.is_some() {
                    // dorado :   another-redis-desktop-manager
                    println!("installer {:?}", installer.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
                let bin = x64.bin;
                if bin.is_some() {
                    // dorado : fasttracker2-clone.json
                    println!("bin {:?}", bin.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
                let extract_dir = x64.extract_dir;
                if extract_dir.is_some() {
                    //  lemon : abstreet.json
                    println!("extract_dir {:?}", extract_dir.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
                let uninstaller = x64.uninstaller;
                if uninstaller.is_some() {
                    //  DEV-tools  :lagarith-lossless-video-codec.json
                    println!("uninstaller {:?}", uninstaller.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
                let shortcuts = x64.shortcuts;
                if shortcuts.is_some() {
                    // dorado  :  crystaldiskinfo-aoi-edition.json
                    println!("shortcuts {:?}", shortcuts.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
                let checkver = x64.checkver;
                if checkver.is_some() {
                    let checkver = checkver.unwrap();
                    println!("checkver {:?}", checkver);
                    println!(" path {}", path.display());
                    return true;
                }
                let pre_install = x64.pre_install;
                if pre_install.is_some() {
                    // cmontage : abricotine.json
                    println!("pre_install {:?}", pre_install.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
                let post_install = x64.post_install;
                if post_install.is_some() {
                    // extras  :  rstudio.json
                    println!("post_install {:?}", post_install.unwrap());
                    println!(" path {}", path.display());
                    return true;
                }
            }
        }
        false
    }
    #[allow(unused)]
    #[ignore]
    fn find_env_add_path(manifest: InstallManifest, path: PathBuf, count: &Arc<Mutex<i32>>) -> u8 {
        let env_add_path = &manifest.env_add_path;
        if env_add_path.is_some() {
            let env_add_path = env_add_path.clone();
            if env_add_path.is_some() {
                let env_add_path = env_add_path.unwrap();
                *count.lock().unwrap() += 1;
                if *count.lock().unwrap() >= 10 {
                    return 1;
                }
                println!("env_add_path {:?}", env_add_path);
                println!(" path {}", path.display());
            }
        }
        0
    }

    #[allow(unused)]
    fn find_env_set(manifest: InstallManifest, path: PathBuf, count: &Arc<Mutex<i32>>) -> bool {
        let env_set = &manifest.env_set;
        if env_set.is_some() {
            let env_set = env_set.clone().unwrap();
            let pretty_str = serde_json::to_string_pretty(&env_set).unwrap();
            println!("env_set {:?}", pretty_str);
            *count.lock().unwrap() += 1;
            if *count.lock().unwrap() >= 10 {
                return true;
            }
        }
        false
    }
}
