use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quickjs::QuickJS;

pub fn criterion_benchmark(c: &mut Criterion) {
    let quickjs = QuickJS::default();
    let script = include_str!("../../../track_points.js");
    let data = include_str!("../../../track_points.json");
    c.bench_function("try_execute", |b| {
        b.iter(|| {
            black_box(
                quickjs
                    .try_execute(script, Some(data), false, false)
                    .unwrap(),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
