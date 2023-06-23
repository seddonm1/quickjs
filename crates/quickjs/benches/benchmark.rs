use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quickjs::{QuickJS, TimeLimit};

pub fn bench(c: &mut Criterion) {
    let script = include_str!("../../../track_points.js");
    let data = include_str!("../../../track_points.json");

    let quickjs = QuickJS::try_new(None, false, false, None, None).unwrap();
    c.bench_function("try_execute", |b| {
        b.iter(|| black_box(quickjs.try_execute(script, Some(data)).unwrap()))
    });

    let quickjs = QuickJS::try_new(
        None,
        false,
        false,
        None,
        Some(TimeLimit {
            time_limit: Duration::from_millis(10000),
            evaluation_interval: Duration::from_millis(100),
        }),
    )
    .unwrap();
    c.bench_function("try_execute_with_time_limit", |b| {
        b.iter(|| black_box(quickjs.try_execute(script, Some(data)).unwrap()))
    });
}

criterion_group!(group, bench);
criterion_main!(group);
