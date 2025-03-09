use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use crate::manifest::manifest_deserialize::*  ;
#[allow(unused)]
#[macro_export]
macro_rules! arch_specific_field {
    ($self:ident, $field:ident) => {{
        let mut ret = $self.inner.$field.as_ref();

        if let Some(arch) = $self.inner.architecture.as_ref() {
            if cfg!(target_arch = "x86") {
                if let Some(ia32) = &arch.ia32 {
                    let $field = ia32.$field.as_ref();
                    if $field.is_some() {
                        ret = $field;
                    }
                }   
            }

            if cfg!(target_arch = "x86_64") {
                if let Some(amd64) = &arch.amd64 {
                    let $field = amd64.$field.as_ref();
                    if $field.is_some() {
                        ret = $field;
                    }
                }
            }

            if cfg!(target_arch = "aarch64") {
                if let Some(aarch64) = &arch.aarch64 {
                    let $field = aarch64.$field.as_ref();
                    if $field.is_some() {
                        ret = $field;
                    }
                }
            }
        }
        ret
    }};
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallManifest {
  #[serde(skip)]
  bucket: Option<String>,
  #[serde(skip)]
  name: Option<String>,

  /// 版本号
  pub version: Option<String>,

  ///  与installer相同的选项，但运行文件/脚本来卸载应用程序。在scoop uninstall和scoop update期间调用
  pub uninstaller: Option<ManifestObj>,

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
  pub installer: Option<ManifestObj>,

  ///  指定在开始菜单中可用的快捷方式值
  /**
  该数组必须包含可执行文件/标签对。第三和第四个元素是可选的。
   目标文件的路径[必需]
   快捷方式的名称（支持子目录： <AppsSubDir>\\<AppShortcut>例如sysinternals ）[必需]
   启动参数[可选]
   图标文件的路径[可选]
  */
  pub shortcuts: Option<ArrayOrDoubleDimensionArray >,

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
  pre_install 、 post_install 、 shortcuts 、 uninstaller 、 url和msi [ msi已弃用]）*/
  pub architecture: Option<ManifestObj> ,
  ///定义如何自动更新清单。
   /**
    "autoupdate": {
       "architecture": {
           "64bit": {
               "url": "https://www.7-zip.org/a/7z$cleanVersion-x64.msi"
           },
           "32bit": {
               "url": "https://www.7-zip.org/a/7z$cleanVersion.msi"
           },
           "arm64": {
               "url": "https://www.7-zip.org/a/7z$cleanVersion-arm64.exe"
           }
       }
   }*/
  pub autoupdate: Option<ManifestObj>,

  pub cookie: Option<HashMap<String, Option<serde_json::Value>>>,
  //将自动安装的应用程序的运行时依赖项。另请参阅suggest （如下）
  pub depends: Option<ManifestObj>,
  pub description: Option<String>,
  //应用程序维护人员和开发人员可以使用bin/checkver工具来检查应用程序的更新版本
  // 。清单中的checkver属性是一个正则表达式，可用于匹配应用程序主页中应用程序的当前稳定版本
  pub checkver: Option<ObjectOrString  >, // 用于检查更新的配置
 /**
 显示一条消息，建议提供补充功能的可选应用程序。请参阅ant的示例。
   ["Feature Name"] = [ "app1", "app2"... ]
   例如 "JDK": [ "extras/oraclejdk", "openjdk" ]
   如果已经安装了针对该功能建议的任何应用程序，则该功能将被视为“已完成”，并且用户将不会看到任何建议。
 */
  pub suggest : Option<ManifestObj  > ,
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
  应用程序维护人员和开发人员可以使用bin/checkver工具来检查应用程序的更新版本。
  清单中的checkver属性是一个正则表达式，可用于匹配应用程序主页中应用程序的当前稳定版本
  。有关示例，请参阅go清单。如果主页没有可靠地指示当前版本，您还可以指定不同的 URL 来检查 -
  有关示例，请参阅ruby​​ 清单。*/
  pub checksum: Option<ManifestObj>,  //文件的校验和
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
  pub url: Option<ArrayOrString>,

  //字符串或字符串数组，其中包含url中每个 URL 的文件哈希值。默认情况下，
  // 哈希值是 SHA256，但您可以通过在哈希字符串前添加“sha512:”、“sha1:”或“md5:”前缀来使用 SHA512、SHA1 或 MD5
  pub hash: Option<ArrayOrString>,
  //   如果url指向压缩文件（支持 .zip、.7z、.tar、.gz、.lzma 和 .lzh），Scoop 将仅提取其中指定的目录
  pub extract_dir: Option<String>,
  // 如果url指向压缩文件（支持 .zip、.7z、.tar、.gz、.lzma 和 .lzh），Scoop 会将所有内容提取到指定目录
  pub extract_to: Option<ArrayOrString >,
  ///  主页 URL
  pub homepage: Option<String>,
  //将此目录添加到用户路径（如果使用--global则添加到系统路径）。
  // 该目录是相对于安装目录的，并且必须位于安装目录内。
  pub env_add_path: Option<ArrayOrString>, // 添加到 PATH 环境变量的路径。
  //如果安装程序基于 InnoSetup，则设置为布尔值true
  pub innosetup: Option<bool>,
  /**
  序的软件许可证的字符串或哈希值。对于知名许可证，请使用https://spdx.org/licenses中找到的标识符。
  对于其他许可证，请使用许可证的 URL（如果有）。否则，请酌情使用“免费软件”、“专有软件”、“公共领域”、
  “共享软件”或“未知”（定义如下）。如果不同的文件有不同的许可证，请用逗号（,）分隔许可证。
  如果整个应用程序是双重许可的，则使用管道符号 (|) 分隔许可证。
    identifier ：SPDX 标识符，或“免费软件”（永久免费使用）、“专有”（必须付费使用）、“公共领域”、
  “共享软件”（免费试用，最终必须付费）或“未知”（无法确定许可证），视情况而定。
    url ：对于非 SPDX 许可证，请包含许可证的链接。也可以包含 SPDX 许可证的链接。
  */
  pub license: Option<ObjectOrString>,
  /// 包含注释的单行字符串或字符串数组
  #[serde(rename = "##")]
  manifest_comment : Option<ArrayOrString>,
  //：单行字符串或字符串数组，其中包含在安装应用程序后显示的消息。
  pub notes: Option<ArrayOrString >,

  //保存在应用程序的数据目录中的目录和文件的字符串或字符串数组。持久数据 , 二维数组定义目录别名
  pub persist: Option<StringOrArrayOrDoubleDimensionArray>,

  //作为 PowerShell 模块安装在~/scoop/modules中。
  //name （ psmodule必需）：模块的名称，该名称应与解压目录中的至少一个文件匹配，以便 PowerShell 将其识别为模块。
  pub psmodule: Option<ManifestObj >,

  /// 为用户（或系统，如果使用--global ）设置一个或多个环境变量
  pub env_set: Option<ManifestObj >,
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
  pub pre_install: Option<ArrayOrString>,  // 安装前执行的命令
  /// 安装应用程序后要执行的命令的一行字符串或字符串数组。这些可以使用$dir 、 $persist_dir和$version等变量
  pub post_install: Option<ArrayOrString>, // 安装后执行的命令
  pub pre_uninstall: Option<ArrayOrString>, // 卸载前执行的命令
  pub post_uninstall: Option<ArrayOrString >, // 卸载后执行的命令

}




impl InstallManifest {



}
