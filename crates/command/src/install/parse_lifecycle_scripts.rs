use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_old_scoop_dir, get_scoop_cfg_path,
    init_scoop_global, init_user_scoop,
};
use crate::install::InstallOptions::ArchOptions;
use crate::install::{install_app, InstallOptions};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{InstallerUninstallerStruct, StringArrayOrString};
use crate::utils::system::get_system_default_arch;
use anyhow::{bail, Context};
use crossterm::style::Stylize;
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LifecycleScripts {
    PreInstall,
    PostInstall,
    Installer,
    Uninstaller,
    PreUninstall,
    PostUninstall,
}

pub fn check_7zip_installed() -> anyhow::Result<()> {
    let output = Command::new("7z")
        .arg("i")
        .output()
        .expect("Failed to execute 7z command");
    if !output.status.success() {
        bail!("7zip is not installed. Please install it and try again.")
    } else {
        let output_str = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output_str.contains("7z.dll") {
            bail!("7zip is not installed correctly. Please install it and try again.")
        }
    }
    Ok(())
}

pub fn parse_lifecycle_scripts(
    scripts: LifecycleScripts,
    manifest_path: &str,
    options: &[InstallOptions],
    app_name: &str,
    installed_arch: Option<&str>,
) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(manifest_path)
        .expect("Failed to read manifest file on parse_lifecycle_scripts");
    let manifest_obj: InstallManifest = serde_json::from_str(&content)
        .expect("Failed to parse manifest file on parse_lifecycle_scripts");

    let version = manifest_obj.version;
    if version.is_none() {
        bail!("Manifest file does not have version")
    }

    let global = if options.contains(&InstallOptions::Global) {
        true
    } else {
        false
    };
    let app_version = version.unwrap();
    let install_arch =
        if let Some(ArchOptions(arch)) = options.iter().find(|opt| matches!(opt, ArchOptions(_))) {
            Ok(arch.to_string()) as anyhow::Result<String>
        } else if installed_arch.is_some() {
            Ok(installed_arch.unwrap().to_string())
        } else {
            Ok(get_system_default_arch()?)
        }
        .expect("Failed to get system default architecture");
    let architecture = manifest_obj.architecture;
    match scripts {
        LifecycleScripts::PreInstall => {
            let pre_install = if let Some(pre_install) = manifest_obj.pre_install {
                Some(pre_install)
            } else if architecture.is_some() {
                let architecture = architecture.unwrap();
                match install_arch.as_str() {
                    "64bit" => {
                        let special_arch_props = architecture.get_specific_architecture("64bit");
                        if special_arch_props.is_some() {
                            let x64 = special_arch_props.unwrap();
                            if let Some(pre_install) = x64.clone().pre_install {
                                Some(pre_install)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "32bit" => {
                        let special_arch_props = architecture.get_specific_architecture("32bit");
                        if special_arch_props.is_some() {
                            let x86 = special_arch_props.unwrap();
                            if let Some(pre_install) = x86.clone().pre_install {
                                Some(pre_install)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "arm64" => {
                        let special_arch_props = architecture.get_specific_architecture("arm64");
                        if special_arch_props.is_some() {
                            let arm64 = special_arch_props.unwrap();
                            if let Some(pre_install) = arm64.clone().pre_install {
                                Some(pre_install)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            if pre_install.is_some() {
                let pre_install = pre_install.unwrap();
                invoke_ps_scripts(
                    pre_install,
                    "pre_install",
                    app_name,
                    app_version.as_str(),
                    install_arch.as_str(),
                    global,
                    content.as_str(),
                )
                .expect("Failed to execute pre_install script");
            }
        }
        LifecycleScripts::PostInstall => {
            let post_install = if let Some(post_install) = manifest_obj.post_install {
                Some(post_install)
            } else if architecture.is_some() {
                let architecture = architecture.unwrap();
                match install_arch.as_str() {
                    "64bit" => {
                        let special_arch_props = architecture.get_specific_architecture("64bit");
                        if special_arch_props.is_some() {
                            let x64 = special_arch_props.unwrap();
                            if let Some(post_install) = x64.clone().post_install {
                                Some(post_install)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "32bit" => {
                        let special_arch_props = architecture.get_specific_architecture("32bit");
                        if special_arch_props.is_some() {
                            let x86 = special_arch_props.unwrap();
                            if let Some(post_install) = x86.clone().post_install {
                                Some(post_install)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "arm64" => {
                        let special_arch_props = architecture.get_specific_architecture("arm64");
                        if special_arch_props.is_some() {
                            let arm64 = special_arch_props.unwrap();
                            if let Some(post_install) = arm64.clone().post_install {
                                Some(post_install)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            if post_install.is_some() {
                let post_install = post_install.unwrap();
                invoke_ps_scripts(
                    post_install,
                    "post_install",
                    app_name,
                    app_version.as_str(),
                    install_arch.as_str(),
                    global,
                    content.as_str(),
                )
                .expect("Failed to execute post_install script");
            }
        }
        LifecycleScripts::Installer => {
            let installer = if let Some(installer) = manifest_obj.installer {
                Some(installer)
            } else if architecture.is_some() {
                let architecture = architecture.unwrap();
                match install_arch.as_str() {
                    "64bit" => {
                        let special_arch_props = architecture.get_specific_architecture("64bit");
                        if special_arch_props.is_some() {
                            let x64 = special_arch_props.unwrap();
                            if let Some(installer) = x64.clone().installer {
                                Some(installer)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "32bit" => {
                        let special_arch_props = architecture.get_specific_architecture("32bit");
                        if special_arch_props.is_some() {
                            let x86 = special_arch_props.unwrap();
                            if let Some(installer) = x86.clone().installer {
                                Some(installer)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "arm64" => {
                        let special_arch_props = architecture.get_specific_architecture("arm64");
                        if special_arch_props.is_some() {
                            let arm64 = special_arch_props.unwrap();
                            if let Some(installer) = arm64.clone().installer {
                                Some(installer)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            if installer.is_some() {
                let installer = installer.unwrap();
                installer_uninstaller_parser(
                    installer,
                    &install_arch,
                    "installer",
                    app_name,
                    app_version.as_str(),
                    global,
                    content.as_str(),
                )
                .expect("Failed to execute installer script");
            }
        }
        LifecycleScripts::Uninstaller => {
            let uninstaller = if let Some(uninstaller) = manifest_obj.uninstaller {
                Some(uninstaller)
            } else if architecture.is_some() {
                let architecture = architecture.unwrap();
                match install_arch.as_str() {
                    "64bit" => {
                        let special_arch_props = architecture.get_specific_architecture("64bit");
                        if special_arch_props.is_some() {
                            let x64 = special_arch_props.unwrap();
                            if let Some(uninstaller) = x64.clone().uninstaller {
                                Some(uninstaller)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "32bit" => {
                        let special_arch_props = architecture.get_specific_architecture("32bit");
                        if special_arch_props.is_some() {
                            let x86 = special_arch_props.unwrap();
                            if let Some(uninstaller) = x86.clone().uninstaller {
                                Some(uninstaller)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "arm64" => {
                        let special_arch_props = architecture.get_specific_architecture("arm64");
                        if special_arch_props.is_some() {
                            let arm64 = special_arch_props.unwrap();
                            if let Some(uninstaller) = arm64.clone().uninstaller {
                                Some(uninstaller)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            if uninstaller.is_some() {
                let uninstaller = uninstaller.unwrap();
                installer_uninstaller_parser(
                    uninstaller,
                    &install_arch,
                    "uninstaller",
                    app_name,
                    app_version.as_str(),
                    global,
                    content.as_str(),
                )
                .expect("Failed to execute uninstaller script");
            }
        }
        LifecycleScripts::PreUninstall => {
            let pre_uninstall = if let Some(pre_uninstall) = manifest_obj.pre_uninstall {
                Some(pre_uninstall)
            } else {
                None
            };
            if pre_uninstall.is_some() {
                let pre_uninstall = pre_uninstall.unwrap();
                invoke_ps_scripts(
                    pre_uninstall,
                    "pre_uninstall",
                    app_name,
                    app_version.as_str(),
                    install_arch.as_str(),
                    global,
                    content.as_str(),
                )
                .expect("Failed to execute pre_uninstall script");
            }
        }
        LifecycleScripts::PostUninstall => {
            let post_uninstall = if let Some(post_uninstall) = manifest_obj.post_uninstall {
                Some(post_uninstall)
            } else {
                None
            };
            if post_uninstall.is_some() {
                let post_uninstall = post_uninstall.unwrap();
                invoke_ps_scripts(
                    post_uninstall,
                    "post_uninstall",
                    app_name,
                    app_version.as_str(),
                    install_arch.as_str(),
                    global,
                    content.as_str(),
                )
                .expect("Failed to execute post_uninstall script");
            }
        }
    }
    Ok(())
}

fn installer_uninstaller_parser(
    installer: InstallerUninstallerStruct,
    cpu_arch: &str,
    script_type: &str,
    app_name: &str,
    app_version: &str,
    global: bool,
    manifest_str: &str,
) -> anyhow::Result<()> {
    let scripts = installer.script;
    let file = installer.file;
    let keep = installer.keep;
    let args = installer.args;
    if file.is_some() || args.is_some() {
        let current_dir = if global {
            get_app_current_dir_global(app_name)
        } else {
            get_app_current_dir(app_name)
        };
        let program_name = current_dir.clone() + file.unwrap().as_str();
        if !is_in_dir(Path::new(&current_dir), Path::new(&program_name)) {
            abort(&format!(
                "Error in manifest: {} {:?} is outside the app directory.",
                title_case(script_type),
                program_name
            ));
        } else if !Path::new(&program_name).exists() {
            abort(&format!(
                "{} {:?} is missing.",
                title_case(script_type),
                program_name
            ));
        }

        let mut substitutions = HashMap::new();
        substitutions.insert("$dir", current_dir.as_str());
        substitutions.insert("$global", if global { "true" } else { "false" });
        substitutions.insert("$version", app_version);

        // Substitute arguments
        let fn_args = if let Some(args) = args {
            let args = match args {
                StringArrayOrString::StringArray(arr) => arr.join(" "),
                StringArrayOrString::Null => "".into(),
                StringArrayOrString::String(str) => str,
            };
            substitute(&args, &substitutions, false)?
        } else {
            String::new()
        };

        // Execute based on file type
        let prog_path = Path::new(&program_name);
        let uninstall = if script_type == "uninstaller" {
            true
        } else {
            false
        };
        if prog_path.extension().map_or(false, |ext| ext == "ps1") {
            // Execute PowerShell script
            let status = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(format!("& '{}' {}", prog_path.display(), fn_args))
                .status()?;

            if !status.success() {
                handle_execution_failure(uninstall, app_name)?;
            }
        } else {
            // Execute external command
            let status = Command::new(&prog_path)
                .args(fn_args.split_whitespace())
                .status()?;

            if !status.success() {
                handle_execution_failure(uninstall, app_name)?;
            }

            let keep = keep.unwrap_or(false);
            if !keep {
                std::fs::remove_file(&prog_path).context(format!(
                    "Failed to remove program file {}",
                    prog_path.display()
                ))?;
            }
        }
    }
    if scripts.is_some() {
        let script = scripts.unwrap();
        invoke_ps_scripts(
            script,
            script_type,
            app_name,
            app_version,
            cpu_arch,
            global,
            manifest_str,
        )
        .expect("Failed to execute installer/uninstaller script");
    }
    Ok(())
}

fn handle_execution_failure(uninstall: bool, app_name: &str) -> Result<(), anyhow::Error> {
    if uninstall {
        bail!("Uninstallation aborted.")
    } else {
        bail!(format!(
            "Installation aborted. You might need to run 'scoop uninstall {}' before trying again.",
            app_name
        ))
    }
}

fn substitute(
    entity: &str,
    params: &HashMap<&str, &str>,
    regex_escape: bool,
) -> Result<String, anyhow::Error> {
    let mut result = entity.to_string();

    for (key, value) in params {
        if regex_escape {
            let re = Regex::new(&regex::escape(key))?;
            result = re.replace_all(&result, *value).to_string();
        } else {
            result = result.replace(key, value);
        }
    }

    Ok(result)
}

fn invoke_ps_scripts(
    scripts: StringArrayOrString,
    script_type: &str,
    app_name: &str,
    app_version: &str,
    cpu_arch: &str,
    global: bool,
    manifest_str: &str,
) -> anyhow::Result<()> {
    let result = check_7zip_installed();
    let options = if global {
        vec![InstallOptions::Global]
    } else {
        vec![]
    };
    if result.is_err() {
        eprintln!("{}: {}", "Error".red().bold(), result.unwrap_err());
        install_app("7zip", options.as_slice())?;
    }
    print!(
        "{}",
        format!("Running {script_type} lifecycle script......")
            .dark_green()
            .bold()
            .to_string()
    );
    std::io::stdout().flush()?; // flush  stdout buffer
    let scripts = match scripts {
        StringArrayOrString::String(scripts) => scripts,
        StringArrayOrString::StringArray(scripts) => scripts.join("\r\n"),
        StringArrayOrString::Null => String::new(),
    };

    let core_script = include_str!("../../../../asset_scripts/core.ps1");
    let decompress_script = include_str!("../../../../asset_scripts/decompress.ps1");
    let manifest_script = include_str!("../../../../asset_scripts/manifest.ps1");
    let temp = std::env::temp_dir();
    let core_path = temp.join("core.ps1");
    let decompress_path = temp.join("decompress.ps1");
    let manifest_path = temp.join("manifest.ps1");
    let temp_str = temp.to_str().unwrap();
    if !core_path.exists() {
        std::fs::write(&core_path, core_script).context(format!(
            "Failed to write core.ps1 file {} at line 522",
            core_path.display()
        ))?;
    }
    if !decompress_path.exists() {
        std::fs::write(&decompress_path, decompress_script).context(format!(
            "Failed to write decompress file {} at line 556",
            decompress_path.display()
        ))?;
    }
    if !manifest_path.exists() {
        std::fs::write(&manifest_path, manifest_script).context(format!(
            "Failed to write manifest file {} at line 568",
            manifest_path.display()
        ))?;
    }
    let old_scoop_dir = get_old_scoop_dir();
    let cfg_path = get_scoop_cfg_path();
    // ! @''@用于转义Json字符串中的单引号 
    let manifest_obj = format!(
        "$json =  @'\n{}\n'@;
        $manifest = $json | ConvertFrom-Json; $manifest | ConvertTo-Json -Depth 10;",
        manifest_str
    );

    let scoop_home = if global {
        init_scoop_global()
    } else {
        init_user_scoop()
    };
    let global_dir = init_scoop_global();
    let injects_var = format!(
        r#"
      $app = "{app_name}" ;
      $architecture = "{cpu_arch}";
      $version = "{app_version}" ;
      $cmd ="install" ;
      $global = ${global}  ;
      $scoopdir ="{scoop_home}" ;
      $dir = "{scoop_home}\apps\$app\current" ;
      $globaldir  = "{global_dir}";
      $oldscoopdir  = "{old_scoop_dir}" ;
      $original_dir = "{scoop_home}\apps\$app\$version";
      $modulesdir  = "{scoop_home}\modules";
      $cachedir  =  "{scoop_home}\cache";
      $bucketsdir  = "{scoop_home}\buckets";
      $persist_dir  = "{scoop_home}\persist\$app";
      $cfgpath   ="{cfg_path}" ;
      $urls = @(script:url $manifest $architecture);
      $fname = $urls.ForEach({{ url_filename $_ }});
  "#
    );

    let include_header = format!(
        r#". "{temp_str}core.ps1";
. "{temp_str}decompress.ps1";
. "{temp_str}manifest.ps1"
 "#
    );
    let ps_script = format!(
        r#"
{}
{}
{}
{}
"#,
        include_header, manifest_obj, injects_var, scripts
    );
    // println!("script: {}", &ps_script);
    let output = Command::new("powershell.exe")
        .args(&["-NoProfile", "-Command"])
        .arg(ps_script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("Failed to execute powershell script");
    if output.status.success() {
        println!("✅!")
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌\n{}", stderr);
        bail!("Failed to execute powershell script")
    }
    Ok(())
}

fn abort(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1);
}

// 检查 prog 是否在 path 路径内
fn is_in_dir(base: &Path, prog: &Path) -> bool {
    match prog.canonicalize() {
        Ok(prog_abs) => match base.canonicalize() {
            Ok(base_abs) => prog_abs.starts_with(&base_abs),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

mod test_parse_lifecycle_scripts {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn include_ps() {
        const ROOT: &str = env!("CARGO_MANIFEST_DIR");
        let _core = [ROOT, "../../asset_scripts/core.ps1"].concat();
        let root = std::env::current_dir().unwrap();

        println!("ROOT: {}", root.display());
        let temp = std::env::temp_dir();
        let temp_str = temp.to_str().unwrap();
        let include_header = format!(
            r#". "{temp_str}core.ps1";
.  "{temp_str}decompress.ps1"; "#
        );
        println!("INCLUDE HEADER:\n{}", include_header);
    }

    #[test]
    fn test_parser() {
        let str = StringArrayOrString::Null;
        invoke_ps_scripts(
            str,
            "test",
            "app",
            "1.1.1",
            "manifest_str",
            true,
            "manifest_str",
        )
        .unwrap()
    }
    #[test]
    fn test_7z_check() {
        check_7zip_installed().unwrap();
    }

    #[test]
    fn test_parse_gdu() {
        let manifest_path = Path::new(r"A:\Scoop\buckets\main\bucket\gdu.json");
        // let str = "Rename-Item \"$dir\\$($fname -replace '\\.zip$')\" 'gdu.exe'";
        let manifest_str = std::fs::read_to_string(manifest_path).unwrap();
        let manifest_obj: InstallManifest = serde_json::from_str(&manifest_str).unwrap();
        let pre_install = manifest_obj.pre_install.unwrap_or_default();
        let str = match pre_install {
            StringArrayOrString::StringArray(_) => String::new(),
            StringArrayOrString::Null => String::new(),
            StringArrayOrString::String(script) => script,
        };
        let str = StringArrayOrString::String(str.into());
        invoke_ps_scripts(
            str,
            "pre_install",
            "gdu",
            "5.30.1",
            "64bit",
            false,
            &manifest_str,
        )
        .unwrap();
    }
}
