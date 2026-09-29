use crate::i18n::tr;
use crate::tr_fmt;
use anyhow::{Context, Result};
use bat::PrettyPrinter;
use crossterm::style::Stylize;
use serde_json::Value;
use std::path::Path;

pub fn get_user_config_path() -> String {
    let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home_dir = std::env::var("USERPROFILE")
            .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        format!("{}\\.config\\scoop\\config.json", home_dir)
    });
    config_path
}

fn read_config(config_path: &Path) -> Result<Value> {
    if !config_path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let content = std::fs::read_to_string(config_path).context(format!(
        "Failed to read config file '{}'",
        config_path.display()
    ))?;
    if content.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let val: Value = serde_json::from_str(&content).context(format!(
        "Failed to parse config file '{}' as JSON",
        config_path.display()
    ))?;
    if !val.is_object() {
        anyhow::bail!(
            "Config file '{}' root must be a JSON object",
            config_path.display()
        );
    }
    Ok(val)
}

fn write_config(config_path: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(config_path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

pub fn display_all_config() {
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let content_bytes = std::fs::read(config_path).unwrap_or_default();
        if content_bytes.is_empty() {
            println!("{{}}");
            return;
        }
        let _ = PrettyPrinter::new()
            .input_from_bytes(content_bytes.as_slice())
            .language("json")
            .print();
    } else {
        let _ = write_config(config_path, &Value::Object(serde_json::Map::new()));
        println!("{{}}");
    }
}

pub fn get_all_config() -> Value {
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    read_config(config_path).unwrap_or_else(|e| {
        log::error!("Failed to read config: {}", e);
        Value::Object(serde_json::Map::new())
    })
}

pub fn get_config_value(name: &str) {
    let name = name.to_lowercase();
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    let config_json = match read_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{}: {}",
                tr("Failed to read configuration file", "读取配置文件失败")
                    .dark_red()
                    .bold(),
                e
            );
            return;
        }
    };
    if let Some(value) = config_json.get(&name) {
        match value {
            Value::String(s) => {
                println!("{}", s.to_owned().dark_yellow().bold());
            }
            Value::Null => {
                println!(
                    "{}",
                    tr_fmt!(
                        "Config key '{name}' does not exist",
                        "{name}\t配置项不存在",
                        name = name
                    )
                    .dark_red()
                    .bold()
                );
            }
            Value::Bool(b) => {
                println!("{}", b.to_string().dark_yellow().bold());
            }
            Value::Number(n) => {
                println!("{}", n.to_string().dark_yellow().bold());
            }
            Value::Array(arr) => {
                if let Ok(str) = serde_json::to_string_pretty(&arr) {
                    println!("{}", str.dark_yellow().bold());
                }
            }
            Value::Object(obj) => {
                if let Ok(str) = serde_json::to_string_pretty(&obj) {
                    println!("{}", str.dark_yellow().bold());
                }
            }
        }
    } else {
        println!(
            "{}",
            tr_fmt!(
                "Config key '{name}' does not exist",
                "{name}\t配置项不存在",
                name = name
            )
            .dark_red()
            .bold()
        );
    }
}

pub fn get_config_value_no_print(name: &str) -> String {
    let name = name.trim().to_lowercase();
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    let config_json = match read_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to read config file: {}", e);
            return String::new();
        }
    };
    if let Some(value) = config_json.get(&name) {
        match value {
            Value::String(s) => s.to_owned(),
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Array(arr) => serde_json::to_string_pretty(&arr).unwrap_or_default(),
            Value::Object(obj) => serde_json::to_string_pretty(&obj).unwrap_or_default(),
        }
    } else {
        String::new()
    }
}

pub fn set_config_value(name: &str, value: &str) {
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    let mut config_json = match read_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{}: {}",
                tr("Failed to read configuration file", "读取配置文件失败")
                    .dark_red()
                    .bold(),
                e
            );
            return;
        }
    };
    if let Some(obj) = config_json.as_object_mut() {
        obj.insert(name.to_string(), Value::String(value.to_string()));
    }
    if let Err(e) = write_config(config_path, &config_json) {
        eprintln!(
            "{}: {}",
            tr("Failed to save configuration file", "保存配置文件失败")
                .dark_red()
                .bold(),
            e
        );
        return;
    }
    println!(
        "{}",
        tr_fmt!(
            "Successfully set '{name}' to '{value}'",
            "{name} 设置成功为 {value}",
            name = name.green().bold(),
            value = value.dark_yellow().bold()
        )
    );
}

pub fn remove_config_value(name: &str) {
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    let mut config_json = match read_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{}: {}",
                tr("Failed to read configuration file", "读取配置文件失败")
                    .dark_red()
                    .bold(),
                e
            );
            return;
        }
    };
    if let Some(obj) = config_json.as_object_mut() {
        if !obj.contains_key(name) {
            eprintln!(
                "{}",
                tr_fmt!(
                    "Config key '{name}' does not exist",
                    "{name} 配置不存在",
                    name = name
                )
                .dark_red()
                .bold()
            );
            return;
        } else {
            obj.remove(name);
            if let Err(e) = write_config(config_path, &config_json) {
                eprintln!(
                    "{}: {}",
                    tr("Failed to save configuration file", "保存配置文件失败")
                        .dark_red()
                        .bold(),
                    e
                );
                return;
            }
            println!(
                "{}",
                tr_fmt!(
                    "'{name}' has been deleted",
                    "{name} 已被删除",
                    name = name.dark_red().bold()
                )
            );
        }
    } else {
        eprintln!(
            "{}",
            tr("Config file is empty", "config文件配置为空")
                .dark_red()
                .bold()
        );
    }
}
