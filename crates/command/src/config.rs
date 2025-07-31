use bat::PrettyPrinter;
use crossterm::style::Stylize;
use serde_json::Value;
use std::path::Path;

pub fn get_user_config_path() -> String {
    let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home_dir = std::env::var("USERPROFILE").unwrap();
        format!("{}\\.config\\scoop\\config.json", home_dir)
    });
    config_path
}

pub fn display_all_config() {
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let content_bytes = std::fs::read(config_path).unwrap();
        PrettyPrinter::new()
            .input_from_bytes(content_bytes.as_slice())
            .language("json")
            .print()
            .unwrap();
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::File::create_new(config_path).unwrap();
    }
}

pub fn get_all_config() -> Value {
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let config_file = std::fs::File::open(config_path).unwrap();
        let config_json: Value = serde_json::from_reader(config_file).unwrap();
        config_json
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::File::create_new(config_path).unwrap();
        println!("{}", "配置文件不存在".dark_red().bold());
        Value::Null
    }
}

pub fn get_config_value(name: &str) {
    let name = name.to_lowercase();
    let config_path = get_user_config_path();
    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let config_file = std::fs::File::open(config_path).unwrap();
        let config_json: Value = serde_json::from_reader(config_file).unwrap();
        if let Some(value) = config_json.get(&name) {
            match value {
                Value::String(s) => {
                    println!("{}", s.to_owned().dark_yellow().bold());
                }
                Value::Null => {
                    println!("{}", format!("{name}\t配置项不存在").dark_red().bold());
                }
                Value::Bool(b) => {
                    println!("{}", b.to_string().dark_yellow().bold());
                }
                Value::Number(n) => {
                    println!("{}", n.to_string().dark_yellow().bold());
                }
                Value::Array(arr) => {
                    let str = serde_json::to_string_pretty(&arr).unwrap();
                    println!("{}", str.dark_yellow().bold());
                }
                Value::Object(obj) => {
                    let str = serde_json::to_string_pretty(&obj).unwrap();
                    println!("{}", str.dark_yellow().bold());
                }
            }
        } else {
            println!("{}", format!("{name}\t配置项不存在").dark_red().bold());
        }
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::File::create_new(config_path).unwrap();
        println!("{}", "配置文件不存在".dark_red().bold());
    }
}

pub fn get_config_value_no_print(name: &str) -> String {
    let name = name.trim().to_lowercase();
    let config_path = get_user_config_path();

    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let config_file = std::fs::File::open(config_path).unwrap();
        let content = std::fs::read_to_string(config_path).unwrap();
        if content.is_empty() {
            return String::new();
        }
        let config_json: Value = serde_json::from_reader(config_file).unwrap();
        if let Some(value) = config_json.get(name) {
            match value {
                Value::String(s) => s.to_owned(),
                Value::Null => String::new(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::Array(arr) => {
                    let str = serde_json::to_string_pretty(&arr).unwrap();
                    str
                }
                Value::Object(obj) => serde_json::to_string_pretty(&obj).unwrap(),
            }
        } else {
            String::new()
        }
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::File::create_new(config_path).unwrap();
        println!("{}", "配置文件不存在".dark_red().bold());
        String::new()
    }
}

pub fn set_config_value(name: &str, value: &str) {
    let config_path = get_user_config_path();

    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let config_file = std::fs::File::open(config_path).unwrap();
        let mut config_json: Value = serde_json::from_reader(config_file).unwrap();
        if let Some(obj) = config_json.as_object_mut() {
            obj.insert(name.to_string(), Value::String(value.to_string()));
        }
        let file = std::fs::File::create(config_path).unwrap();
        serde_json::to_writer_pretty(file, &config_json).unwrap();
        println!(
            "{} 设置成功为 {}",
            name.green().bold(),
            value.dark_yellow().bold()
        );
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::File::create_new(config_path).unwrap();
        println!("{}", "配置文件不存在".dark_red().bold());
    }
}

pub fn remove_config_value(name: &str) {
    let config_path = get_user_config_path();

    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let config_file = std::fs::File::open(config_path).unwrap();
        let mut config_json: Value = serde_json::from_reader(config_file).unwrap();
        if let Some(obj) = config_json.as_object_mut() {
            if !obj.contains_key(name) {
                eprintln!("{}", format!("{name} 配置不存在").dark_red().bold());
                return;
            } else {
                obj.remove(name);
                let file = std::fs::File::create(config_path).unwrap();
                serde_json::to_writer_pretty(file, &config_json).unwrap();
                println!("{} 已被删除", name.dark_red().bold());
            }
        } else {
            eprintln!("{}", "config文件配置为空".dark_red().bold());
        }
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::File::create_new(config_path).unwrap();
        println!(
            "{}",
            "配置文件不存在,检查 $env:USERPROFILE/.config/scoop/config.json 文件"
                .dark_red()
                .bold()
        );
    };
}
