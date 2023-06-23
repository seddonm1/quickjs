use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quickjs::{QuickJS, TimeLimit};

pub fn try_execute(c: &mut Criterion) {
    let quickjs = QuickJS::try_new(None, false, false, None, None).unwrap();
    let script = include_str!("../../../track_points.js");
    let data = include_str!("../../../track_points.json");
    c.bench_function("try_execute", |b| {
        b.iter(|| black_box(quickjs.try_execute(script, Some(data)).unwrap()))
    });
}

pub fn try_execute_with_time_limit(c: &mut Criterion) {
    let quickjs = QuickJS::try_new(
        None,
        false,
        false,
        None,
        Some(TimeLimit {
            time_limit: Duration::from_millis(1000),
            evaluation_frequency: Duration::from_millis(10),
        }),
    )
    .unwrap();
    let script = include_str!("../../../track_points.js");
    let data = include_str!("../../../track_points.json");
    c.bench_function("try_execute_with_time_limit", |b| {
        b.iter(|| black_box(quickjs.try_execute(script, Some(data)).unwrap()))
    });
}

criterion_group!(group, try_execute, try_execute_with_time_limit);
criterion_main!(group);
