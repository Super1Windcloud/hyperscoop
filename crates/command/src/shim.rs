use std::path::Path;
use anyhow::bail;
use crossterm::style::Stylize;
use crate::init_env::{get_shims_path, get_shims_path_global};
use crate::init_hyperscoop;

pub fn  list_all_shims (global : bool)  -> Result<(), anyhow::Error> {
  let shim_path = if  global { get_shims_path_global()} else { get_shims_path()};
  let shim_path = Path::new(&shim_path); 
  if!shim_path.exists()  {
     bail!( "{} is not exist", shim_path.display() );
  }
  let mut shims = vec![];
   for entry in shim_path.read_dir()? {
      let entry = entry?;
      let path = entry.path();
     if !path.is_file() { continue; }
      let file_name = path.file_name().unwrap().to_str().unwrap();
       if  !file_name.ends_with(".shim")  { continue; }
     let shim_name = file_name.replace(".shim","");
     let  shim_path =  shim_path.to_str().unwrap().to_owned() + "\\" +&shim_name +".exe";
      let  shim_source = std::fs::read_to_string(path)?.replace("path =","");
     shims.push( (shim_name, shim_path, shim_source) );
   }

   for (name, path, source) in shims {
     println!( "Names: {:<15}  Path: {:<30} \nSource: {:<30}" , name, path, source);
   }
  
  Ok(())
}

pub fn list_shims_by_regex (regex : String ) {
   log::info!("list_shims_by_regex");
  let shim_path = init_hyperscoop().unwrap().get_shims_path();
  let shim_path = Path::new(&shim_path);
  let mut shims = vec![];
  for entry in shim_path.read_dir().unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if !path.is_file() { continue; }
    let file_name = path.file_name().unwrap().to_str().unwrap();
    if  !file_name.ends_with(".shim")  { continue; }
    let shim_name = file_name.replace(".shim","");
     let re = regex::Regex::new(&regex).unwrap();
     let cap = re.captures(&shim_name);
    if   cap.is_none() {  continue; }
    let  shim_path =  shim_path.to_str().unwrap().to_owned() + "\\" +&shim_name +".exe";
    let  shim_source = std::fs::read_to_string(path).unwrap().replace("path =","");
    shims.push( (shim_name, shim_path, shim_source) );
  }

  for (name, path, source) in shims {
    println!( "Names: {:<15}  Path: {:<30} \nSource: {:<30}" , name, path, source);
  }
}

pub fn list_shim_info (name : String, global : bool) -> anyhow::Result<()>{
    let   shim_name = name.clone();
  let shim_path = if  global { get_shims_path_global()} else { get_shims_path()};
  let shim_path = Path::new(&shim_path);
  if !shim_path.exists()  {
     bail!( "{} is not exist", shim_path.display() );
  }
  for entry in shim_path.read_dir()? {
    let entry = entry?;
    let path = entry.path();
    if !path.is_file() { continue; }
    let file_name = path.file_name().unwrap().to_str().unwrap();
    if !file_name.contains(&name ) || !file_name.ends_with(".exe")  || !(file_name ==(name.clone()+".exe")) { continue; }
    let shim_path  =path.to_str().unwrap().to_owned(); 
    println!("{:<30} : {}", "Name ".green().bold(), shim_name.dark_green().bold()); 
    println!("{:<30} : {}", "Path ".green().bold(), shim_path.clone().dark_green().bold());
    println!("{:<30} : {}", "Source ".green().bold(), name .dark_green().bold()); 
    println!("{:<30} : {}", "Type ".green().bold(),  "Application".dark_green().bold()); 
    println!("{:<30} : {}", "IsGlobal ".green().bold(), "False".dark_green().bold()); 
    println!("{:<30} : {}", "IsHidden ".green().bold(), "False".dark_green().bold()); 
    
    return   Ok(());
  }
  println!("{}{}", "No shim found for name: ".red().bold(), shim_name.dark_cyan().bold());
  
  Ok(())
}


 pub fn execute_add_shim (name : String, source : String, global : bool) {
  let   shim_name = name.clone();
  let   shim_source = source.clone();
  let   shim_path = if global { get_shims_path_global()} else { get_shims_path()} ; 
  let   shim_file = shim_path.clone() + "\\" + &shim_name + ".shim";
  let   shim_content = format!("path = {}\n", shim_source);
   let shim_file = Path::new(&shim_file);   
   log::info!("Adding shim: {} ", shim_file.display());
    std::fs::write(shim_file, shim_content).unwrap();  
   
   let str = format!(r#"
   use std::process::Command;
  use std::env;

fn main() {{
    let target =  r"{shim_source}" ; 
    let args: Vec<String> = env::args().skip(1).collect();

    let status = Command::new(target)
        .args(&args)
        .status()
        .expect("Failed to execute target program");

    std::process::exit(status.code().unwrap_or(1));
}}
   "#  );  
   let  shim_source_rs = shim_path.clone() + "\\" + &shim_name + ".rs"; 
   std::fs::write(&shim_source_rs, str ).unwrap(); 
   compile_exe(&shim_source_rs);
   
  
}

fn compile_exe( source : &String) {
   let path = Path::new(source);
   let mut cmd = std::process::Command::new("rustc");
   cmd.arg(path);
   cmd.arg("-o");
   let   target_name =  path.with_extension("exe") ;  
   cmd.arg(target_name);
   let status = cmd.status().unwrap();
   if !status.success() {
      println!("{}{}", "Failed to compile source: ".red().bold(), source.clone().dark_cyan().bold());
      std::process::exit(status.code().unwrap_or(1)); 
   } 
  std::fs::remove_file(path).unwrap();  
  let  source_pdb = path.with_extension("pdb");  
  log::info!("PDB: {}", source_pdb.display());
  std::fs::remove_file(source_pdb).unwrap();   
}

pub  fn  alter_shim_source (name : String, source : String ) {  
  let   shim_name = name.clone();
  let   shim_source = source.clone();
  let   shim_path = init_hyperscoop().unwrap().get_shims_path();
  let   shim_file = shim_path.clone() + "\\" + &shim_name + ".shim";
  let   shim_content = format!("path = {}\n", shim_source);
   let shim_file = Path::new(&shim_file);   
   log::info!("Altering shim: {} ", shim_file.display());
    std::fs::write(shim_file, shim_content).unwrap();
}