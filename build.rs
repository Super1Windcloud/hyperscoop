use std::env;



fn main() {  
   let  lang =  env::var("LANG").or(env::var("LC_ALL"))
     .or(env::var("LC_CTYPE")).unwrap_or_default(); 
   let  lang_prefix  = lang.split("_").next().unwrap_or("en");  
  
   println!("cargo:rustc-env=BUILD_SYSTEM_LANG={}" ,lang_prefix); 
  
   if  lang_prefix =="zh" {
      println!("cargo:rustc-cfg=system_lang_zh");
   }
}


