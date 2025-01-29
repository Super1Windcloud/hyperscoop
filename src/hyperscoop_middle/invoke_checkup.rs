

pub fn execute_checkup_command()  -> Result<(), anyhow::Error>{
  let results = run_checkup(); 
  log::info!("Checkup results: {:#?}", results);
  print_results(&results); 
  Ok(()) 
}
use std::{
  env,
  fs::{self, File},
  io::{ Write},
  net::TcpStream,
  path::PathBuf,
  process::Command,
};

// 检查结果结构体
#[derive(Debug)]
struct CheckResult {
  name: String,
  status: &'static str,
  message: String,
  suggestion: String,
}

impl CheckResult {
  fn new(
    name: String,
    status: &'static str,
    message: String,
    suggestion: String,
  ) -> Self {
    Self {
      name,
      status,
      message,
      suggestion,
    }
  }
}

// 主检查函数
fn run_checkup() -> Vec<CheckResult> {
  let mut results = Vec::new();

  // 检查环境变量
  results.push(check_env_vars());

  // 检查依赖项
  results.push(check_dependency("git"));
  results.push(check_dependency("7z"));

  // 检查Scoop目录权限
  results.push(check_scoop_permissions());

  // 网络检查
  results.push(check_network());

  // 其他检查可以继续添加...

  results
}

// 环境变量检查
fn check_env_vars() -> CheckResult {
  let mut message = String::new();
  let mut suggestion = String::new();
  let mut status = "OK";

  // 检查SCOOP环境变量
  if let Ok(scoop_path) = env::var("SCOOP") {
    message.push_str(&format!("SCOOP路径: {}\n", scoop_path));
  } else {
    status = "FAIL";
    message.push_str("未找到SCOOP环境变量\n");
    suggestion.push_str("请通过`scoop config SCOOP $HOME/scoop`设置\n");
  }

  // 检查PATH是否包含shims目录
  let shims_path = PathBuf::from(env::var("SCOOP").unwrap_or_default()).join("shims");
  if let Ok(path) = env::var("PATH") {
    if !path.contains(&shims_path.to_string_lossy().to_string())  {
      status = "FAIL";
      message.push_str("PATH未包含Scoop的shims目录\n");
      suggestion.push_str("请将以下路径添加到PATH:\n");
      suggestion.push_str(&format!("{}\n", shims_path.display()));
    }
  }

  CheckResult::new("环境变量".parse().unwrap(), status, message, suggestion)
}

// 依赖检查
fn check_dependency(tool: &str) -> CheckResult {
  let output = Command::new(tool).arg("--version").output();

  if output.is_ok() {
    CheckResult::new(
      format!("{} 依赖", tool),
      "OK",
      format!("{} 已安装", tool),
      String::new(),
    )
  } else { 
    CheckResult::new(
      format!("{} 依赖", tool), 
      "FAIL",
      format!("未找到 {}", tool),
      format!("请执行 `scoop install {tool}`"),
    )
  }
}

// 目录权限检查
fn check_scoop_permissions() -> CheckResult {
  let scoop_path = match env::var("SCOOP") {
    Ok(p) => PathBuf::from(p),
    Err(_) => return CheckResult::new(
      "目录权限".to_string(),
      "FAIL",
      "无法确定Scoop目录".into(),
      "请先设置SCOOP环境变量".into(),
    ),
  };

  // 尝试创建测试文件
  let test_file = scoop_path.join("permission_test.tmp");
  match File::create(&test_file).and_then(|mut f| f.write_all(b"test")) {
    Ok(_) => {
      fs::remove_file(test_file).ok();
      CheckResult::new("目录权限".to_string(), "OK", "写入权限正常".into(), String::new())
    }
    Err(e) => CheckResult::new(
      "目录权限".to_string(),
      "FAIL",
      format!("无法写入文件: {}", e),
      "请以管理员权限运行或修改目录权限".into(),
    ),
  }
}

// 网络检查
fn check_network() -> CheckResult {
  match TcpStream::connect(("github.com", 80)) {
    Ok(_) => CheckResult::new(
      "网络连接".to_string(),
      "OK",
      "可以访问 GitHub".into(),
      String::new(),
    ),
    Err(e) => CheckResult::new(
      "网络连接".to_string(),
      "FAIL",
      format!("无法连接 GitHub: {}", e),
      "请检查网络或代理设置".into(),
    ),
  }
}

// 输出格式化
fn print_results(results: &[CheckResult]) {
  println!("{:-<40}", "");
  println!("{:20} | {:5} | {}", "检查项", "状态", "详细信息");
  println!("{:-<40}", "");

  for result in results {
    let status = match result.status {
      "OK" => "\x1b[32mOK\x1b[0m",
      _ => "\x1b[31mFAIL\x1b[0m",
    };

    println!(
      "{:20} | {:5} | {}",
      result.name, status, result.message
    );

    if !result.suggestion.is_empty() {
      println!("{:>20}   {}", "建议：", result.suggestion);
    }
  }
}

 