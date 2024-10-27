 use  std:: { env  , process } ;
fn main() {
     let args : Vec<String> = env::args().collect();
     println!("{:?}", args);
    if   (args.len() < 2)  {
         println!("Please provide a command to execute");
    }
      process::exit(0);
    if   (args[1] == "linux" || args[1] == "Linux"  ) {
         are_you_on_linux();
    } else if  (args[1] == "windows" || args[1] == "Windows" ) {
         println!("You are running windows!") ;
    }
}


 #[cfg(target_os = "windows")]
 fn are_you_on_linux() {
     println!("You are running linux!") ;
     if cfg!(target_os = "windows") {
         println!("You are running windows!") ;
     }
 }