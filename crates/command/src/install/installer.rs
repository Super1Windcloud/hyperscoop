use serde_json::Value;
use crate::manifest::manifest_deserialize::{ArrayOrString, ManifestObj};

pub fn   show_suggest (suggest : &ManifestObj){
  let pretty_json = serde_json::to_string_pretty(suggest).unwrap();
  println!("建议安装以下依赖包 : {}", pretty_json);

}

pub fn show_notes (  notes : &ArrayOrString) { 
  match notes {
    ArrayOrString::StringArray(notes) => {
      for note in notes {
        println!("{}", note);
      }
    },
    ArrayOrString::String(note) => {
      println!("{}", note);
    }
    ArrayOrString::Null => {}
  }
}

pub fn handle_arch (arch : ManifestObj) {
  let pretty_json = serde_json::to_string_pretty(&arch ).unwrap();
  // println!("建议安装以下依赖包 : {}", pretty_json);
}