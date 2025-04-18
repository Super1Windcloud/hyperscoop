// Below is the code of /tests/ui.rs for the test use:
use  clap:: {command, arg, value_parser}  ;



fn cmd() -> clap::Command {
  command!() // requires `cargo` feature
    .arg(
      arg!(<PORT>)
        .help("Network port to use")
        .value_parser(value_parser!(usize)),
    )
}

fn main() { 
   let args = cmd().get_matches(); 
    
}