use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Check if we're in shim creation mode
  if let Some(path_arg) = env::args().nth(1) {
    if path_arg == "--path" {
      if let Some(target_path) = env::args().nth(2) {
        return create_path_shim_file(&target_path);
      } else {
        eprintln!("Usage: {} --path <target_executable_path>", env::args().next().unwrap_or_default());
        std::process::exit(1);
      }
    }
  }

  // Otherwise run normally as a shim
  let exit_code = run()?;
  std::process::exit(exit_code);
}

pub fn create_path_shim_file (target_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  // Get the name for the shim from the target executable
  let target_name = Path::new(target_path)
    .file_stem()
    .and_then(|s| s.to_str())
    .ok_or("Invalid target executable name")?;

  // Create the shim content
  let content = format!("path = \"{}\"", target_path);

  // Determine the shim file name
  let shim_name = format!("{}.shim", target_name);
  let shim_path = env::current_dir()?.join(&shim_name);

  // Write the shim file
  let mut file = File::create(&shim_path)?;
  file.write_all(content.as_bytes())?;
  println!("Created shim file: {}", shim_path.display());

  // On Windows, we can also create an exe with the same name
  #[cfg(windows)]
  {
    let exe_name = format!("{}.exe", target_name);
    let current_exe = env::current_exe()?;

    // Copy the current executable to act as the shim
    fs::copy(current_exe, env::current_dir()?.join(exe_name))?;
    println!("Created shim executable");
  }

  Ok(())
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
  let exe_path = env::current_exe()?;
  let dir = exe_path.parent().ok_or("No parent directory")?;
  let name = exe_path
    .file_stem()
    .and_then(|s| s.to_str())
    .ok_or("Invalid executable name")?;

  let config_path = dir.join(format!("{}.shim", name));
  if !config_path.exists() {
    eprintln!(
      "Couldn't find {} in {}",
      config_path.file_name().unwrap_or_default().to_string_lossy(),
      dir.display()
    );
    return Ok(1);
  }

  let config = read_config(&config_path)?;
  let path = config.get("path").map(|s| s.as_str()).unwrap_or("");
  if path.is_empty() {
    eprintln!("No 'path' specified in shim config");
    return Ok(1);
  }

  let add_args = config.get("args").map(|s| s.as_str()).unwrap_or("");
  let cmd_args = get_command_line_args(add_args)?;

  // First try to execute directly
  match execute_directly(path, &cmd_args) {
    Ok(exit_code) => Ok(exit_code),
    Err(e) => {
      // If direct execution fails, try with shell execute
      if is_elevation_required(&e) {
        match execute_with_shell(path, &cmd_args) {
          Ok(exit_code) => Ok(exit_code),
          Err(e) => {
            eprintln!("Failed to execute {}: {}", path, e);
            Ok(1)
          }
        }
      } else {
        Err(e.into())
      }
    }
  }
}

fn execute_directly(program: &str, args: &str) -> Result<i32, std::io::Error> {
  let mut cmd = Command::new(program);

  if !args.is_empty() {
    cmd.args(args.split_whitespace());
  }

  cmd.stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit());

  let status = cmd.status()?;
  Ok(status.code().unwrap_or(1))
}

fn execute_with_shell(program: &str, args: &str) -> Result<i32, std::io::Error> {
  #[cfg(windows)]
  let status = if !args.is_empty() {
    Command::new("cmd")
      .args(&["/C", &format!("{} {}", program, args)])
      .stdin(Stdio::inherit())
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status()?
  } else {
    Command::new("cmd")
      .args(&["/C", program])
      .stdin(Stdio::inherit())
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status()?
  };

  #[cfg(not(windows))]
  let status = if !args.is_empty() {
    Command::new("sh")
      .args(&["-c", &format!("{} {}", program, args)])
      .stdin(Stdio::inherit())
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status()?
  } else {
    Command::new("sh")
      .args(&["-c", program])
      .stdin(Stdio::inherit())
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status()?
  };

  Ok(status.code().unwrap_or(1))
}

fn is_elevation_required(error: &std::io::Error) -> bool {
  #[cfg(windows)]
  {
    error.raw_os_error() == Some(740) // ERROR_ELEVATION_REQUIRED
  }
  #[cfg(not(windows))]
  {
    false
  }
}

fn read_config(path: &Path) -> Result<HashMap<String, String>, std::io::Error> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let mut config = HashMap::new();

  for line in reader.lines() {
    let line = line?;
    if let Some((key, value)) = line.split_once('=') {
      config.insert(key.trim().to_string(), value.trim().to_string());
    }
  }

  Ok(config)
}

fn get_command_line_args(add_args: &str) -> Result<String, std::io::Error> {
  let mut args = env::args().skip(1); // Skip program name

  let mut combined_args = String::new();

  if !add_args.is_empty() {
    combined_args.push_str(add_args);
  }

  for arg in args {
    if !combined_args.is_empty() {
      combined_args.push(' ');
    }
    combined_args.push_str(&arg);
  }

  Ok(combined_args)
}



mod test_shim {

  #[test]
  fn test_create_shim() {
    use crate::create_path_shim_file;
    use std::process::Command;  
    let  target_path = r"A:\Scoop\apps\zigmod\current\zigmod.exe";
    create_path_shim_file(target_path).unwrap(); 
     let  workspace= std::env::current_dir().unwrap();
    let  shim_exe=  workspace.join("shim.exe");   
    let   zigmod = workspace.join("zigmod.exe");
     println!("shim exe is {:?}", shim_exe);
     std::fs::copy(shim_exe, &zigmod).unwrap(); 
    let result = Command::new(zigmod); 
      
  }
}