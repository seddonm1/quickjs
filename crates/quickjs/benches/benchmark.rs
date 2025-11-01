use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use quickjs::{QuickJSBuilder, TimeLimit};
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let script = format!(
        "const data={};\n{}",
        include_str!("../../../track_points.json"),
        include_str!("../../../track_points.js")
    );

    let quickjs = QuickJSBuilder::new().build().unwrap();
    c.bench_function("try_execute", |b| {
        b.iter(|| black_box(quickjs.try_execute(&script).unwrap()))
    });

    let quickjs = QuickJSBuilder::new().with_memory_limit(4194304).build().unwrap();
    c.bench_function("try_execute_with_memory_limit", |b| {
        b.iter(|| black_box(quickjs.try_execute(&script).unwrap()))
    });

    let quickjs = QuickJSBuilder::new()
        .with_time_limit(
            TimeLimit::new(Duration::from_millis(10000)).with_evaluation_interval(Duration::from_micros(100)),
        )
        .build()
        .unwrap();
    c.bench_function("try_execute_with_time_limit_100us", |b| {
        b.iter(|| black_box(quickjs.try_execute(&script).unwrap()))
    });

    let quickjs = QuickJSBuilder::new()
        .with_time_limit(
            TimeLimit::new(Duration::from_millis(10000)).with_evaluation_interval(Duration::from_micros(1000)),
        )
        .build()
        .unwrap();
    c.bench_function("try_execute_with_time_limit_1000us", |b| {
        b.iter(|| black_box(quickjs.try_execute(&script).unwrap()))
    });

    let quickjs = QuickJSBuilder::new()
        .with_time_limit(
            TimeLimit::new(Duration::from_millis(10000)).with_evaluation_interval(Duration::from_micros(10000)),
        )
        .build()
        .unwrap();
    c.bench_function("try_execute_with_time_limit_10000us", |b| {
        b.iter(|| black_box(quickjs.try_execute(&script).unwrap()))
    });
}

criterion_group!(group, bench);
criterion_main!(group);
