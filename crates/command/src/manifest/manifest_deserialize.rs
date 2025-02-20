use serde::{Deserialize, Serialize};
pub type ManifestObj = serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)] // 允许处理多种类型
pub enum ArrayOrString {
    StringArray(Vec<String>), // 数组类型
    #[default]
    Null,
    String(String), // 字符串类型
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectOrArray {
    #[default]
    Null,
    ManifestObj(serde_json::Value), // 对象类型
    StringArray(Vec<String>),       // 数组类型
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectOrString {
    #[default]
    Null,
    ManifestObj(serde_json::Value), // 对象类型
    String(String),                 // 字符串类型
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ArrayOrDoubleDimensionArray   {
    #[default]
    Null,
    StringArray(Vec<String>),
    DoubleDimensionArray(Vec<Vec<String>>), 
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOrDoubleDimensionArray {
    #[default]
    Null,
    String(String), // 字符串类型
    StringArray(Vec<String>),
    DoubleDimensionArray(Vec<Vec<String>>),
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOrDotDimensionArrayOrObject  {
    #[default]
    Null,
    ManifestObj(serde_json::Value), 
    String(String), // 字符串类型
    StringArray(Vec<String>),
    DoubleDimensionArray(Vec<Vec<String>>),
}

mod tests {
    #[test]
    fn test_serde_json() {
        let json = r#"
       { 
         "name": "hp",
         "version": "1.0.0",
         "architecture": {
           "amd64": {
             "installer": "innosetup"
           }
         }, 
         "suggest" : { 
         "JDK": "1.8+"
         }, 
         "depends" : {  
         "JDK":   ["8", "9","22" ,"GraalVM"] 
         }
       }
     "#;
      let manifest   :serde_json::Value =serde_json::from_str(json).unwrap();   
      let  suggest = manifest["suggest"].as_object().unwrap(); 
      let  depends = manifest["depends"].as_object().unwrap();  
      println!("suggest {:?}" , suggest);
      println!("depends {:?}" , depends.values().collect::<Vec<_>>());
    }
}
