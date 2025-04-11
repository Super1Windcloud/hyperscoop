use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("parse url", |b| {
    b.iter_batched(
      || "https://gitee.com/SuperWindcloud/hyperscoop",
      |url| black_box(url::Url::parse(black_box(url)).unwrap()),
      BatchSize::SmallInput,
    );
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
