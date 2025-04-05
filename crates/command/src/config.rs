use std::path::Path;
use crossterm::style::Stylize;

pub  fn  display_all_config() { 
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  } );  
  let config_path =  Path::new(&config_path);
  if config_path.exists() {
    log::info!("{}", config_path.display()); 
    let config_file = std::fs::File::open(config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    let max_key_length = config_json
      .as_object()
      .unwrap()
      .keys()
      .map(|key| key.len())
      .max()
      .unwrap_or(0);
    for (key, value) in config_json.as_object().unwrap() {
      let padded_key = format!("{:width$}", key, width = max_key_length);
      println!(
        "{}\t:\t{}",
        padded_key.green().bold(),
        value.to_string().yellow().bold()
      );
    }
  }
}


pub  fn  get_all_config() -> serde_json::Value {
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  } );  
  let config_path =  Path::new(&config_path);
  if config_path.exists() {
    let config_file = std::fs::File::open(config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    return config_json;
  } 
  serde_json::Value::Null 
}


pub fn  get_config_value (name : &str) ->String  {
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  } );  
  let config_path =  Path::new(&config_path);
  if config_path.exists() {
    let config_file = std::fs::File::open(config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    if let Some(value) = config_json.get(name) {
      println!("{}", value.to_string().yellow().bold()); 
      value.as_str().unwrap().to_string()
    } else {
      println!("{}", "配置项不存在".red().bold());
      String::new()
    }
  } else {
    println!("{}", "配置文件不存在".red().bold());
    String::new()
  }
}

pub fn  get_config_value_no_print  (name : &str) ->String  {
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  } );  
  let config_path =  Path::new(&config_path);
  if config_path.exists() {
    let config_file = std::fs::File::open(config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    if let Some(value) = config_json.get(name) {
      value.as_str().unwrap().to_string()
    } else {
      String::new()
    }
  } else {
    String::new()
  }
}

pub fn  set_config_value (name : &str, value : &str) {
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  } );  
  let config_path =  Path::new(&config_path);
  if config_path.exists() {
    let config_file = std::fs::File::open(config_path).unwrap();
    let mut config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    if let Some(obj) = config_json.as_object_mut() {
      obj.insert(name.to_string(), serde_json::Value::String(value.to_string()));
    }
    let file = std::fs::File::create(config_path).unwrap();
    serde_json::to_writer_pretty(file, &config_json).unwrap();
    println!("{} 设置成功为 {}", name.green().bold(), value.yellow().bold());
  } else {
    println!("{}", "配置文件不存在".red().bold());
  }
} 

pub fn  remove_config_value (name : &str) {
  let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
    let home_dir = std::env::var("USERPROFILE").unwrap();
    format!("{}\\.config\\scoop\\config.json", home_dir)
  } );  
  let config_path =  Path::new(&config_path);
  if config_path.exists() {
    let config_file = std::fs::File::open(config_path).unwrap();
    let mut config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    if let Some(obj) = config_json.as_object_mut() {
      obj.remove(name);
    }
    let file = std::fs::File::create(config_path).unwrap();
    serde_json::to_writer_pretty(file, &config_json).unwrap();
    println!("{} 已被删除", name.dark_red().bold());
  } else {
    println!("{}", "配置文件不存在,检查 $env:USERPROFILE/.config/scoop/config.json 文件".dark_red().bold()); 
  } ; 
} 

