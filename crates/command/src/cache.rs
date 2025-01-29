use crossterm::style::Stylize;
use crate::HyperScoop;

pub   fn  display_all_cache_info() {
     log::info!("display_all_cache_info ") ;
   let hyperscoop = HyperScoop::new();
   let cache_dir = hyperscoop.get_cache_path();
   log::info!("cache dir : {:?}", cache_dir);
   let cache_files = std::fs::read_dir(cache_dir).unwrap();
  let   mut infos = Vec::new();
  let   mut count = 0;
   for file in cache_files {
    let path = file.unwrap();
     let  path1 =path.path().clone().file_name().unwrap().to_str().unwrap().to_string();
     let path2 =path.path().clone().to_string_lossy().to_string();
     let  app_name =path1.split("#").collect::<Vec<&str>>()[0].to_string();
     let  zip_size=(std::fs::metadata(&path2).unwrap().len() as f64)/1024f64/1024f64 ; 
       log::info!("cache file : {}",&app_name);
      log::info!("cache file : {}",&path2);
     log::info!("cache size : {} MB",&zip_size);
     let  version = path1.split("#").collect::<Vec<&str>>()[1].to_string();
     log::info!("cache version : {}",&version);
      infos.push((app_name,version,zip_size));
     count += 1;
   }
  let total_size = infos.iter().fold(0f64, |acc, x| acc + x.2); 
  let  total_size_parsed = format!("{:.2}", total_size);
  println!("{} {} {} {} {}\n" , "Total : ".to_string().yellow().bold() ,count.to_string().dark_yellow().bold(), 
     "Files, ".to_string().yellow().bold() ,
           total_size_parsed.to_string().yellow().bold() , "MB".to_string().dark_yellow().bold()) ;
  println!("{:<30}\t\t{:<30}\t\t{:<30}" , "Name".green().bold() , "Version".green().bold(), "Size".green().bold());
  println!("{:<30}\t\t{:<30}\t\t{:<30}" , "____".green().bold() , "_______".green().bold(), "____".green().bold());

  println_cache_info(&infos);


}

fn println_cache_info(app_name : &Vec<(String, String, f64)>) {
  for info in app_name { 
    let zip_size_parsed = format!("{:.2}", info.2);
    println!("{:<15} {:<15} {:<15}", info.0, info.1, zip_size_parsed +" MB");
  }
}
