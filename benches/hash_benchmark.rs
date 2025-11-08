use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::Rng;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use std::process::Command;

fn generate_random_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.r#gen()).collect()
}

fn sha1_benchmark(c: &mut Criterion) {
    let data = generate_random_data(1024 * 1024); // 1MB数据
    c.bench_function("hash sha1", |b| {
        b.iter(|| {
            let mut hasher = Sha1::new();
            hasher.update(black_box(&data));
            hasher.finalize()
        })
    });
}
fn sha256_bench(c: &mut Criterion) {
    let data = generate_random_data(1024 * 1024); // 1MB数据
    c.bench_function("sha256 1MB", |b| {
        b.iter(|| {
            let mut hasher = Sha256::new();
            hasher.update(black_box(&data));
            hasher.finalize()
        })
    });
}

fn sha512_bench(c: &mut Criterion) {
    let data = generate_random_data(1024 * 1024); // 1MB数据
    c.bench_function("sha512 1MB", |b| {
        b.iter(|| {
            let mut hasher = Sha512::new();
            hasher.update(black_box(&data));
            hasher.finalize()
        })
    });
}

fn benchmark_powershell_sha256(c: &mut Criterion) {
    let data = generate_random_data(1024 * 1024);

    c.bench_function("PS SHA256", |b| {
        b.iter(|| powershell_hash_sync(&data, "SHA256").unwrap())
    });
}
fn benchmark_powershell_sha512(c: &mut Criterion) {
    let data = generate_random_data(1024 * 1024);

    c.bench_function("PS SHA512", |b| {
        b.iter(|| powershell_hash_sync(&data, "SHA512").unwrap())
    });
}

fn benchmark_powershell_sha1(c: &mut Criterion) {
    let data = generate_random_data(1024 * 1024); // 1MB数据

    c.bench_function("PS SHA1", |b| {
        b.iter(|| powershell_hash_sync(&data, "SHA1").unwrap())
    });
}

fn powershell_hash_sync(data: &[u8], algorithm: &str) -> std::io::Result<String> {
    let temp_file = std::env::temp_dir().join("hash_test.bin");
    std::fs::write(&temp_file, data)?;
    let file = temp_file.as_path().to_str().unwrap();
    let script = format!(
        r#"
$env:PSModulePath = "$PSHOME\Modules"; 
(Get-FileHash -Path {file} -Algorithm {algorithm}).Hash; 
"#
    );
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

criterion_group!(
    benches,
    sha256_bench,
    sha512_bench,
    sha1_benchmark,
    benchmark_powershell_sha1,
    benchmark_powershell_sha256,
    benchmark_powershell_sha512
);
criterion_main!(benches);
