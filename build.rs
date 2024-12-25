use std::process::Command;
fn main() {
    let status = Command::new("echo")
        .arg("Running custom build steps...")
        .status();

    match status {
        Ok(status) if status.success() => println!("Command executed successfully"),
        Ok(status) => eprintln!("Command executed with status: {}", status),
        Err(e) => eprintln!("Failed to execute command: {}", e),
    };
}
