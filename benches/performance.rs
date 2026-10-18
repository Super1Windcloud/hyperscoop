use command_util_lib::manifest::manifest::get_all_manifest_files_from_bucket;
use command_util_lib::utils::hash::hash_reader;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::hint::black_box;
use std::io::Read;
use std::path::PathBuf;

fn file_hash(c: &mut Criterion) {
    let temp = tempfile::tempdir().unwrap();
    let mut group = c.benchmark_group("file_sha256");
    for size in [1024 * 1024, 8 * 1024 * 1024] {
        let path = temp.path().join(format!("{size}.bin"));
        std::fs::write(&path, vec![0x5a; size]).unwrap();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("whole_file", size), &path, |b, path| {
            b.iter(|| {
                let mut data = Vec::new();
                std::fs::File::open(path)
                    .unwrap()
                    .read_to_end(&mut data)
                    .unwrap();
                black_box(hex::encode(Sha256::digest(&data)))
            });
        });
        group.bench_with_input(BenchmarkId::new("streaming", size), &path, |b, path| {
            b.iter(|| {
                black_box(hash_reader::<Sha256>(std::fs::File::open(path).unwrap()).unwrap())
            });
        });
    }
    group.finish();
}

fn manifest_lookup(c: &mut Criterion) {
    let temp = tempfile::tempdir().unwrap();
    let mut roots = Vec::new();
    for bucket in ["main", "extras", "versions"] {
        let path = temp.path().join(bucket).join("bucket");
        std::fs::create_dir_all(&path).unwrap();
        for i in 0..1000 {
            std::fs::write(path.join(format!("tool{i:04}.json")), b"{}").unwrap();
        }
        roots.push(path.to_string_lossy().into_owned());
    }
    let legacy = || {
        let paths: Vec<PathBuf> = roots
            .par_iter()
            .flat_map_iter(|root| {
                std::fs::read_dir(root)
                    .unwrap()
                    .filter_map(Result::ok)
                    .filter(|entry| {
                        entry.file_type().unwrap().is_file()
                            && entry.path().extension().is_some_and(|ext| ext == "json")
                    })
                    .map(|entry| entry.path())
                    .collect::<Vec<_>>()
            })
            .collect();
        paths
            .par_iter()
            .filter(|path| path.file_stem().unwrap().to_str().unwrap().to_lowercase() == "tool0500")
            .count()
    };
    assert_eq!(
        legacy(),
        get_all_manifest_files_from_bucket(&roots, "tool0500").len()
    );
    c.bench_function("manifest_3000/scan_all", |b| b.iter(|| black_box(legacy())));
    c.bench_function("manifest_3000/direct", |b| {
        b.iter(|| black_box(get_all_manifest_files_from_bucket(&roots, "tool0500")))
    });
}

criterion_group!(benches, file_hash, manifest_lookup);
criterion_main!(benches);
