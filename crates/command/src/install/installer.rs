use anyhow::bail;
use crossterm::style::Stylize;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
use winreg::RegKey;
use crate::init_env::{get_old_scoop_dir, get_scoop_cfg_path, init_env_path, init_scoop_global_path};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{StringArrayOrString, ManifestObj};

pub fn   show_suggest (suggest : &ManifestObj)  -> anyhow::Result<()> {
  let pretty_json = serde_json::to_string_pretty(suggest)?;
  println!("建议安装以下依赖包 : {} \n", pretty_json.dark_yellow().bold()   );


  Ok(() )
}

pub fn show_notes (  notes : &StringArrayOrString)  -> anyhow::Result<()> {
  match notes {
    StringArrayOrString::StringArray(notes) => {
        println!(    "{}" , "Notes : " .to_string().dark_cyan().bold());
        println!(    "{}" , "_____ : " .to_string().dark_cyan().bold());
       for note in notes {
        println!(" {}", note.clone().dark_grey().bold() );
      }
    },
    StringArrayOrString::String(note) => {
      println!("Notes : {}", note.clone() .dark_grey().bold() );
    }
    StringArrayOrString::Null => {}
  }
  Ok(() )
}

pub fn handle_arch (arch : ManifestObj) {
  let pretty_json = serde_json::to_string_pretty(&arch ).unwrap();
  // println!("建议安装以下依赖包 : {}", pretty_json);
}

pub fn handle_env_set (env_set : &ManifestObj, manifest : &InstallManifest) ->   anyhow::Result<()> {
  let app_name = manifest.name.clone().unwrap_or(String::new());
  let app_version = manifest.version.clone().unwrap_or(String::new());
  let scoop_home = init_env_path();
  let global_scoop_home = init_scoop_global_path();

  let app_dir = format!(
    r#"function app_dir($other_app) {{
      return    "{scoop_home}\apps\$other_app\current" ;
  }}"#
  );
  let old_scoop_dir = get_old_scoop_dir();
  let cfg_path = get_scoop_cfg_path();
  let injects_var = format!(
    r#"
      $app = "{app_name}" ;
      $version = "{app_version}" ;
      $cmd ="uninstall" ;
      $global = $false  ;
      $scoopdir ="{scoop_home}" ;
      $dir = "{scoop_home}\apps\$app" ;
      $globaldir  ="{global_scoop_home}";
      $oldscoopdir  = "{old_scoop_dir}" ;
      $original_dir = "{scoop_home}\apps\$app\$version";
      $modulesdir  = "{scoop_home}\modules";
      $cachedir  =  "{scoop_home}\cache";
      $bucketsdir  = "{scoop_home}\buckets";
      $persist_dir  = "{scoop_home}\persist\$app";
      $cfgpath   ="{cfg_path}" ;
  "#
  );

  if let serde_json::Value::Object(env_set) = env_set {
    for (key, env_value ) in env_set {
      let  mut env_value = env_value.to_string().trim().to_string();
      if env_value.is_empty() {
        continue;
      }
      if env_value.contains('/')  {
          env_value = env_value.replace('/', r"\");
      }
      if   env_value.contains(r"\\") {
          env_value = env_value.replace(r"\\", r"\");
      }
      let cmd = format!(r#"Set-ItemProperty -Path "HKCU:\Environment" -Name "{key}" -Value {env_value}"#);


     println!("cmd: {}", cmd);
      let output = std::process::Command::new("powershell")
        .arg("-Command" )
        .arg(&app_dir)
        .arg(&injects_var)
        .arg(cmd)
        .output()?;
      if !output.status.success() {
        let error_output = String::from_utf8_lossy(&output.stderr);
        bail!("powershell failed to remove environment variable: {}", error_output);
      }

       println!("{} {} {}", "env set success : ".to_string().dark_green().bold() , key.to_string().dark_cyan().bold()
                , env_value.dark_cyan().bold() );
    }
  }
  Ok(())
}


pub  fn handle_env_add_path (env_add_path: StringArrayOrString, app_current_dir: String) ->  anyhow::Result<()> {
  let  app_current_dir = app_current_dir.replace('/', r"\");
  if  let StringArrayOrString::StringArray(paths) = env_add_path {
     for  path  in  paths {
         add_bin_to_path(path ,&app_current_dir)? ;
     }
  }else if let StringArrayOrString::String(path) = env_add_path {
         add_bin_to_path(path ,&app_current_dir)? ;
  }

  Ok(())
}
pub  fn  add_bin_to_path ( path : String , app_current_dir :&String ) -> anyhow::Result<()> {
  let  path = path.replace('/', r"\");
  let  path = path.replace('\\', r"\");
  let path =  format!(r"{app_current_dir}\{path}");
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let environment_key = hkcu.open_subkey("Environment")?;
  let user_path: String = environment_key.get_value("PATH")?;

  let  user_path =  format!("{user_path};{path}") ;
  log::trace!("\n 更新后的用户的 PATH: {}", user_path);

  let script = format!(
    r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "User")"#
  );
  let output = std::process::Command::new("powershell")
    .arg("-Command")
    .arg(script)
    .output()?;
  if !output.status.success() {
    bail!("Failed to remove path var");
  }

  Ok(())
}
