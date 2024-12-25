use hscoop_macro::HelloMacro;
pub trait HelloMacro {
    fn hello_macro();
}
#[derive(HelloMacro, Debug)]
struct MyStruct {}

fn main() {
    MyStruct::hello_macro();
}
