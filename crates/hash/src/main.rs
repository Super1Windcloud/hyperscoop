use sha2::Digest;
use sha2::Sha256;
use std::io::{ Read};
use std::process::Command;

fn calculate_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(input).unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    hasher.update(&buffer);

    let sha256 = hasher.finalize();
    format!("{:x}", sha256)
}

fn calculate_sha256_by_pwsh(input: &str) -> String {
    let input = if input.starts_with("\"") {
        input.trim_matches('"').replace("\"", "").to_string()
    } else if input.starts_with("'") {
        input.trim_matches('\'').replace("'", "").to_string()
    } else {
        input.to_string()
    };
    dbg!(&input);
    let script = format!(
        "Get-FileHash -Algorithm SHA256 -Path \"{}\" | Select-Object -ExpandProperty Hash",
        input
    );
    let script = format!(
    "& {{ Get-FileHash -Algorithm SHA256 -LiteralPath '{}' | Select-Object -ExpandProperty Hash }}",
    input.replace('\'', "''")
  );

    dbg!(&script);
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(&script)
        .output()
        .expect("failed to execute process");

    let sha256 = String::from_utf8(output.stdout).unwrap();
    sha256.trim().to_string()
}

fn test_elapsed_time() {
    let start = std::time::Instant::now();
    let scoop = std::env::var("SCOOP").unwrap();
    let cache = std::path::Path::new(&scoop).join("cache");
    for entry in cache.read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            let path = entry.path();
            println!("{:?}", path.display());
            let hash = calculate_sha256_by_pwsh(&entry.path().display().to_string());
            println!("{}", hash);
        }
    }
    let elapsed = start.elapsed();
    println!("PowerShell Elapsed time: {:.2?}", elapsed);

    let start = std::time::Instant::now();
    let scoop = std::env::var("SCOOP").unwrap();
    let cache = std::path::Path::new(&scoop).join("cache");
    for entry in cache.read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            let path = entry.path();
            println!("{:?}", path.display());
            let hash = calculate_sha256(&entry.path().display().to_string());
            println!("{}", hash);
        }
    }
    let elapsed = start.elapsed();
    println!("Rust Sha2 Elapsed time: {:.2?}", elapsed);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <file_path>", args[0]);
        return;
    }
    let file_path = &args[1];
    println!("Calculating SHA256 for file: {}", file_path);
    let hash = calculate_sha256(file_path);
    if hash.is_empty() {
        println!("No SHA256 hash found");
        return;
    }
    println!("{}", hash);
}
