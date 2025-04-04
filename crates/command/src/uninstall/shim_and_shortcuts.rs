use crate::manifest::manifest_deserialize::{
    ArrayOrDoubleDimensionArray, StringOrArrayOrDoubleDimensionArray,
};
use crate::manifest::uninstall_manifest::UninstallManifest;
use crate::utils::system::get_system_default_arch;
use anyhow::bail;
use crossterm::style::Stylize;
use std::path::{Path, PathBuf};

pub fn get_all_shortcuts_link_paths() -> Vec<PathBuf> {
    let paths = vec![
        PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
        PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"),
        PathBuf::from("C:\\Users\\Public\\Desktop"),
        PathBuf::from(format!(
            "C:\\Users\\{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup",
            std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string())
        )),
        PathBuf::from(format!(
            "C:\\Users\\{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs",
            std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string())
        )),
        PathBuf::from(format!(
            "C:\\Users\\{}\\Desktop",
            std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string())
        )),
        PathBuf::from(format!(
            r"C:\Users\{}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Scoop Apps",
            std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string())
        )),
    ];
    paths
}

pub fn rm_start_menu_shortcut(manifest: &UninstallManifest) -> Result<(), anyhow::Error> {
    let shortcuts = manifest.clone().shortcuts;
    let architecture = manifest.clone().architecture;
    if shortcuts.is_none() && architecture.is_none() {
        return Ok(());
    }

    if let Some(shortcut) = shortcuts {
        match shortcut {
            ArrayOrDoubleDimensionArray::Null => return Ok(()),
            ArrayOrDoubleDimensionArray::StringArray(shortcut) => {
                let arg_len = shortcut.len();
                if arg_len < 2 {
                    eprintln!(
                        "{} ",
                        "Failed to find shortcut, maybe manifest json file format error"
                            .dark_yellow()
                            .bold()
                    );
                }
                let shortcut_name = shortcut[1].clone() + ".lnk";
                if shortcut_name.is_empty() {
                    return Ok(());
                }
                let scoop_links = get_all_shortcuts_link_paths();
                for scoop_link in scoop_links {
                    if scoop_link.exists() {
                        let path = scoop_link.join(&shortcut_name);
                        if path.exists() {
                            println!(
                                "Removing start menu shortcut '{}'",
                                path.display().to_string().dark_cyan().bold()
                            );
                            std::fs::remove_file(path)?;
                        }
                    }
                }
            }
            ArrayOrDoubleDimensionArray::DoubleDimensionArray(shortcut) => {
                let arg_len = shortcut.len();
                if arg_len < 1 {
                    eprintln!(
                        "{} ",
                        "Failed to find shortcut, maybe manifest json file format error"
                            .dark_yellow()
                            .bold()
                    );
                }
                for shortcut_item in shortcut {
                    let arg_len = shortcut_item.len();
                    if arg_len < 2 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    let shortcut_name = shortcut_item[1].clone() + ".lnk";
                    if shortcut_name.is_empty() {
                        return Ok(());
                    }
                    let scoop_links = get_all_shortcuts_link_paths();
                    for scoop_link in scoop_links {
                        if scoop_link.exists() {
                            let path = scoop_link.join(&shortcut_name);
                            if path.exists() {
                                println!(
                                    "Removing start menu shortcut '{}'",
                                    path.display().to_string().dark_cyan().bold()
                                );
                                std::fs::remove_file(path)?;
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(architecture) = architecture {
        let system_arch = get_system_default_arch()?;
        let x64 = architecture.x64bit;
        let x86 = architecture.x86bit;
        let arm64 = architecture.arm64;
        if system_arch == "64bit" {
            if x64.is_none() {
                return Ok(());
            }
            let x64 = x64.unwrap();
            let shortcuts = x64.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            match shortcuts {
                ArrayOrDoubleDimensionArray::Null => return Ok(()),
                ArrayOrDoubleDimensionArray::StringArray(shortcut) => {
                    let arg_len = shortcut.len();
                    if arg_len < 2 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    let shortcut_name = shortcut[1].clone() + ".lnk";
                    if shortcut_name.is_empty() {
                        return Ok(());
                    }
                    let scoop_links = get_all_shortcuts_link_paths();
                    for scoop_link in scoop_links {
                        if scoop_link.exists() {
                            let path = scoop_link.join(&shortcut_name);
                            if path.exists() {
                                println!(
                                    "Removing start menu shortcut '{}'",
                                    path.display().to_string().dark_cyan().bold()
                                );
                                std::fs::remove_file(path)?;
                            }
                        }
                    }
                }
                ArrayOrDoubleDimensionArray::DoubleDimensionArray(shortcut) => {
                    let arg_len = shortcut.len();
                    if arg_len < 1 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    for shortcut_item in shortcut {
                        let arg_len = shortcut_item.len();
                        if arg_len < 2 {
                            eprintln!(
                                "{} ",
                                "Failed to find shortcut, maybe manifest json file format error"
                                    .dark_yellow()
                                    .bold()
                            );
                        }
                        let shortcut_name = shortcut_item[1].clone() + ".lnk";
                        if shortcut_name.is_empty() {
                            return Ok(());
                        }
                        let scoop_links = get_all_shortcuts_link_paths();
                        for scoop_link in scoop_links {
                            if scoop_link.exists() {
                                let path = scoop_link.join(&shortcut_name);
                                if path.exists() {
                                    println!(
                                        "Removing start menu shortcut '{}'",
                                        path.display().to_string().dark_cyan().bold()
                                    );
                                    std::fs::remove_file(path)?;
                                }
                            }
                        }
                    }
                }
            }
        } else if system_arch == "32bit" {
            if x86.is_none() {
                return Ok(());
            }
            let x86 = x86.unwrap();
            let shortcuts = x86.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            match shortcuts {
                ArrayOrDoubleDimensionArray::Null => return Ok(()),
                ArrayOrDoubleDimensionArray::StringArray(shortcut) => {
                    let arg_len = shortcut.len();
                    if arg_len < 2 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    let shortcut_name = shortcut[1].clone() + ".lnk";
                    if shortcut_name.is_empty() {
                        return Ok(());
                    }
                    let scoop_links = get_all_shortcuts_link_paths();
                    for scoop_link in scoop_links {
                        if scoop_link.exists() {
                            let path = scoop_link.join(&shortcut_name);
                            if path.exists() {
                                println!(
                                    "Removing start menu shortcut '{}'",
                                    path.display().to_string().dark_cyan().bold()
                                );
                                std::fs::remove_file(path)?;
                            }
                        }
                    }
                }
                ArrayOrDoubleDimensionArray::DoubleDimensionArray(shortcut) => {
                    let arg_len = shortcut.len();
                    if arg_len < 1 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    for shortcut_item in shortcut {
                        let arg_len = shortcut_item.len();
                        if arg_len < 2 {
                            eprintln!(
                                "{} ",
                                "Failed to find shortcut, maybe manifest json file format error"
                                    .dark_yellow()
                                    .bold()
                            );
                        }
                        let shortcut_name = shortcut_item[1].clone() + ".lnk";
                        if shortcut_name.is_empty() {
                            return Ok(());
                        }
                        let scoop_links = get_all_shortcuts_link_paths();
                        for scoop_link in scoop_links {
                            if scoop_link.exists() {
                                let path = scoop_link.join(&shortcut_name);
                                if path.exists() {
                                    println!(
                                        "Removing start menu shortcut '{}'",
                                        path.display().to_string().dark_cyan().bold()
                                    );
                                    std::fs::remove_file(path)?;
                                }
                            }
                        }
                    }
                }
            }
        } else if system_arch == "arm64" {
            if arm64.is_none() {
                return Ok(());
            }
            let arm64 = arm64.unwrap();
            let shortcuts = arm64.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            match shortcuts {
                ArrayOrDoubleDimensionArray::Null => return Ok(()),
                ArrayOrDoubleDimensionArray::StringArray(shortcut) => {
                    let arg_len = shortcut.len();
                    if arg_len < 2 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    let shortcut_name = shortcut[1].clone() + ".lnk";
                    if shortcut_name.is_empty() {
                        return Ok(());
                    }
                    let scoop_links = get_all_shortcuts_link_paths();
                    for scoop_link in scoop_links {
                        if scoop_link.exists() {
                            let path = scoop_link.join(&shortcut_name);
                            if path.exists() {
                                println!(
                                    "Removing start menu shortcut '{}'",
                                    path.display().to_string().dark_cyan().bold()
                                );
                                std::fs::remove_file(path)?;
                            }
                        }
                    }
                }
                ArrayOrDoubleDimensionArray::DoubleDimensionArray(shortcut) => {
                    let arg_len = shortcut.len();
                    if arg_len < 1 {
                        eprintln!(
                            "{} ",
                            "Failed to find shortcut, maybe manifest json file format error"
                                .dark_yellow()
                                .bold()
                        );
                    }
                    for shortcut_item in shortcut {
                        let arg_len = shortcut_item.len();
                        if arg_len < 2 {
                            eprintln!(
                                "{} ",
                                "Failed to find shortcut, maybe manifest json file format error"
                                    .dark_yellow()
                                    .bold()
                            );
                        }
                        let shortcut_name = shortcut_item[1].clone() + ".lnk";
                        if shortcut_name.is_empty() {
                            return Ok(());
                        }
                        let scoop_links = get_all_shortcuts_link_paths();
                        for scoop_link in scoop_links {
                            if scoop_link.exists() {
                                let path = scoop_link.join(&shortcut_name);
                                if path.exists() {
                                    println!(
                                        "Removing start menu shortcut '{}'",
                                        path.display().to_string().dark_cyan().bold()
                                    );
                                    std::fs::remove_file(path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn rm_shim_file(
    shim_path: String,
    manifests: &UninstallManifest,
    app_name: &str,
) -> Result<(), anyhow::Error> {
    let app_name = app_name.to_lowercase() + ".json";
    let shim_path = Path::new(shim_path.as_str());
    let manifest_bin = manifests.clone().bin;
    let architecture = manifests.clone().architecture;
    if manifest_bin.is_none() && architecture.is_none() {
        eprintln!(
            "'{}' ,{}",
            app_name.dark_yellow().bold(),
            "don't have  shim file".dark_yellow().bold(),
        );
        return Ok(());
    }
    if manifest_bin.is_some() {
        match manifest_bin.unwrap() {
            StringOrArrayOrDoubleDimensionArray::String(s) => {
                rm_default_shim_name_file(s, shim_path)?;
            }
            StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
                for item in a {
                    rm_default_shim_name_file(item, shim_path)?;
                }
            }
            StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
                for item in a {
                    let len = item.len();
                    if len == 1 {
                        rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                    }
                    if len == 2 || len == 3 {
                        let exe_name = item[0].clone();
                        let alias_name = item[1].clone();
                        rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                    }
                }
            }
            StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
                for item in a {
                    match item {
                        StringOrArrayOrDoubleDimensionArray::String(s) => {
                            rm_default_shim_name_file(s, shim_path)?;
                        }
                        StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
                            let len = item.len();
                            if len == 1 {
                                rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                            }
                            if len == 2 || len == 3 {
                                let exe_name = item[0].clone();
                                let alias_name = item[1].clone();
                                rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                            }
                        }
                        _ => {
                            println!(" what the fuck bin?   {:?}", item);
                        }
                    }
                }
            }
            _ => {
                bail!("WTF? can't parser this bin object type ")
            }
        }
    }
    if architecture.is_some() {
        let architecture = architecture.unwrap();
        let system_arch = get_system_default_arch()?;
        let x64 = architecture.x64bit;
        let x86 = architecture.x86bit;
        let arm64 = architecture.arm64;
        if system_arch == "64bit" {
            if x64.is_none() {
                return Ok(());
            }
            let x64 = x64.unwrap();
            let bin = x64.bin;
            if bin.is_none() {
                return Ok(());
            }
            let bin = bin.unwrap();
            match bin {
                StringOrArrayOrDoubleDimensionArray::String(s) => {
                    rm_default_shim_name_file(s, shim_path)?;
                }
                StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
                    for item in a {
                        rm_default_shim_name_file(item, shim_path)?;
                    }
                }
                StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
                    for item in a {
                        let len = item.len();
                        if len == 1 {
                            rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                        }
                        if len == 2 || len == 3 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                        }
                    }
                }
                StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
                    for item in a {
                        match item {
                            StringOrArrayOrDoubleDimensionArray::String(s) => {
                                rm_default_shim_name_file(s, shim_path)?;
                            }
                            StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
                                let len = item.len();
                                if len == 1 {
                                    rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                                }
                                if len == 2 || len == 3 {
                                    let exe_name = item[0].clone();
                                    let alias_name = item[1].clone();
                                    rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                                }
                            }
                            _ => {
                                println!(" what the fuck bin?   {:?}", item);
                            }
                        }
                    }
                }
                _ => {
                    bail!("WTF? can't parser this bin object type ")
                }
            }
        } else if system_arch == "32bit" {
            if x86.is_none() {
                return Ok(());
            }
            let x86 = x86.unwrap();
            let bin = x86.bin;
            if bin.is_none() {
                return Ok(());
            }
            let bin = bin.unwrap();
            match bin {
                StringOrArrayOrDoubleDimensionArray::String(s) => {
                    rm_default_shim_name_file(s, shim_path)?;
                }
                StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
                    for item in a {
                        rm_default_shim_name_file(item, shim_path)?;
                    }
                }
                StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
                    for item in a {
                        let len = item.len();
                        if len == 1 {
                            rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                        }
                        if len == 2 || len == 3 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                        }
                    }
                }
                StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
                    for item in a {
                        match item {
                            StringOrArrayOrDoubleDimensionArray::String(s) => {
                                rm_default_shim_name_file(s, shim_path)?;
                            }
                            StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
                                let len = item.len();
                                if len == 1 {
                                    rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                                }
                                if len == 2 || len == 3 {
                                    let exe_name = item[0].clone();
                                    let alias_name = item[1].clone();
                                    rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                                }
                            }
                            _ => {
                                println!(" what the fuck bin?   {:?}", item);
                            }
                        }
                    }
                }
                _ => {
                    bail!("WTF? can't parser this bin object type ")
                }
            }
        } else if system_arch == "arm64" {
            if arm64.is_none() {
                return Ok(());
            }
            let arm64 = arm64.unwrap();
            let bin = arm64.bin;
            if bin.is_none() {
                return Ok(());
            }
            let bin = bin.unwrap();
            match bin {
                StringOrArrayOrDoubleDimensionArray::String(s) => {
                    rm_default_shim_name_file(s, shim_path)?;
                }
                StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
                    for item in a {
                        rm_default_shim_name_file(item, shim_path)?;
                    }
                }
                StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
                    for item in a {
                        let len = item.len();
                        if len == 1 {
                            rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                        }
                        if len == 2 || len == 3 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                        }
                    }
                }
                StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
                    for item in a {
                        match item {
                            StringOrArrayOrDoubleDimensionArray::String(s) => {
                                rm_default_shim_name_file(s, shim_path)?;
                            }
                            StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
                                let len = item.len();
                                if len == 1 {
                                    rm_default_shim_name_file((&item[0]).to_string(), shim_path)?;
                                }
                                if len == 2 || len == 3 {
                                    let exe_name = item[0].clone();
                                    let alias_name = item[1].clone();
                                    rm_alias_shim_name_file(exe_name, alias_name, shim_path)?;
                                }
                            }
                            _ => {
                                println!(" what the fuck bin?   {:?}", item);
                            }
                        }
                    }
                }
                _ => {
                    bail!("WTF? can't parser this bin object type ")
                }
            }
        }
    }
    Ok(())
}

fn rm_alias_shim_name_file(
    exe_name: String,
    alias_name: String,
    shim_path: &Path,
) -> anyhow::Result<()> {
    let mut s = exe_name.clone();
    if s.contains(r"\") {
        let split = s.split(r"\").collect::<Vec<&str>>();
        s = split.last().unwrap().to_string();
    }
    if s.contains("/") {
        let split = s.split(r"/").collect::<Vec<&str>>();
        s = split.last().unwrap().to_string();
    }

    let suffix = s.split(".").last().unwrap();
    let prefix = alias_name.trim();

    let shim_file = shim_path.join(prefix);
    let origin_shim_file = shim_path.join(s.clone());
    if origin_shim_file.exists() {
        println!(
            "origin exe shim file {}",
            origin_shim_file.display().to_string().dark_cyan().bold()
        );
        std::fs::remove_file(origin_shim_file)?;
    }

    if suffix == "exe" {
        let exe_file = prefix.to_string() + ".exe";
        let shim_file = shim_path.join(exe_file);
        if shim_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shim_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shim_file)?;
        }
        let shim = prefix.to_string() + ".shim";
        let shim_file = shim_path.join(shim);
        if !shim_file.exists() {
            return Ok(());
        }
        println!(
            "{} {}",
            "Removing shim file".dark_blue().bold(),
            shim_file.display().to_string().dark_green().bold()
        );
        std::fs::remove_file(shim_file)?;
    }
    if suffix == "bat" || suffix == "cmd" {
        if shim_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shim_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shim_file)?;
        }
        let cmd_str = prefix.to_string() + ".cmd";
        let cmd_file = shim_path.join(cmd_str);

        if cmd_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                cmd_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&cmd_file)?;
        }
    }

    if suffix == "ps1" {
        let ps_file = prefix.to_string() + ".ps1";
        let shim_file = shim_path.join(ps_file);

        if shim_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shim_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shim_file)?;
        }
        let cmd_str = prefix.to_string() + ".cmd";
        let shell_file = shim_path.join(prefix);
        let cmd_file = shim_path.join(cmd_str);
        if shell_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shell_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shell_file)?;
        }
        if cmd_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                cmd_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&cmd_file)?;
        }
    }
    Ok(())
}

fn rm_default_shim_name_file(s: String, shim_path: &Path) -> anyhow::Result<()> {
    let mut s = s.clone();

    if s.contains('\\') {
        let split = s.split(r"\").collect::<Vec<&str>>();
        s = split.last().unwrap().to_string();
    }
    if s.contains('/') {
        let split = s.split(r"/").collect::<Vec<&str>>();
        s = split.last().unwrap().to_string();
    }

    let suffix = s.split(".").last().unwrap();
    let prefix = s.split(".").next().unwrap();
    let shim_file = shim_path.join(s.clone());
    if shim_file.exists() && suffix == "exe" {
        println!(
            "{} {}",
            "Removing shim file".dark_blue().bold(),
            shim_file.display().to_string().dark_green().bold()
        );
        std::fs::remove_file(&shim_file)?;
        let shim = prefix.to_string() + ".shim";
        let shim_file = shim_path.join(shim);
        if !shim_file.exists() {
            return Ok(());
        }
        println!(
            "{} {}",
            "Removing shim file".dark_blue().bold(),
            shim_file.display().to_string().dark_green().bold()
        );
        std::fs::remove_file(shim_file)?;
    }
    if suffix == "bat" || suffix == "cmd" {
        if shim_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shim_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shim_file)?;
        }
        let cmd_str = prefix.to_string() + ".cmd";
        let shell_file = shim_path.join(prefix);
        let cmd_file = shim_path.join(cmd_str);
        if shell_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shell_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shell_file)?;
        }
        if cmd_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                cmd_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&cmd_file)?;
        }
    }

    if shim_file.exists() && suffix == "ps1" {
        println!(
            "{} {}",
            "Removing shim file".dark_blue().bold(),
            shim_file.display().to_string().dark_green().bold()
        );
        std::fs::remove_file(&shim_file)?;

        let cmd_str = prefix.to_string() + ".cmd";
        let shell_file = shim_path.join(prefix);
        let cmd_file = shim_path.join(cmd_str);
        if shell_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                shell_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&shell_file)?;
        }
        if cmd_file.exists() {
            println!(
                "{} {}",
                "Removing shim file".dark_blue().bold(),
                cmd_file.display().to_string().dark_green().bold()
            );
            std::fs::remove_file(&cmd_file)?;
        }
    }

    Ok(())
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_shim_alias_files() {
        let shim_dir = Path::new(r"A:\Scoop\shims");
        rm_alias_shim_name_file("superwindcloud.exe".into(), "superwc".into(), shim_dir).unwrap();
    }

    #[test]
    fn test_bin_path_parser() {
        let mut s = "bin//zig.exe".to_string();
        if s.contains('\\') {
            let split = s.split(r"\").collect::<Vec<&str>>();
            s = split.last().unwrap().to_string();
        }
        if s.contains('/') {
            let split = s.split(r"/").collect::<Vec<&str>>();
            s = split.last().unwrap().to_string();
        }
        println!("s is {s }");
    }
}
