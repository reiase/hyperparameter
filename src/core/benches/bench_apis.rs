use config::Config;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hyperparameter::*;

#[inline(never)]
fn foo(x: i64, y: i64) -> i64 {
    x + y
}

#[inline(never)]
fn foo_with_ps(x: i64) -> i64 {
    with_params! {
        @get y = y or 0;

        x+y
    }
}

#[inline(never)]
fn foo_with_config(x: i64, cfg: &Config) -> i64 {
    let y = cfg.get_int("y").unwrap();
    x + y
}

#[inline(never)]
fn call_foo(nloop: i64) -> i64 {
    let mut sum = 0;
    for i in 0..nloop {
        sum += foo(i, 42);
    }
    sum
}

#[inline(never)]
fn call_foo_with_ps(nloop: i64) -> i64 {
    let mut sum = 0;
    for i in 0..nloop {
        with_params! {
            @set y = 42;

            sum += foo_with_ps(i);
        }
    }
    sum
}

#[inline(never)]
fn call_foo_with_ps_optimized(nloop: i64) -> i64 {
    let mut sum = 0;

    with_params! {
        @set y = 42;

        for i in 0..nloop {
            sum += foo_with_ps(i);
        }
    }
    sum
}

#[inline(never)]
fn call_foo_with_ps_and_raw_btree(nloop: i64) -> i64 {
    let mut sum = 0;
    const KEY: u64 = xxhash("y".as_bytes());
    with_params! {
        @set y = 42;

        for i in 0..nloop {
            sum += THREAD_STORAGE.with(|ts| ts.borrow_mut().get_or_else(KEY, i));
        }
    }
    sum
}

#[inline(never)]
fn call_foo_with_config_rs(nloop: i64, cfg: &Config) -> i64 {
    let mut sum = 0;
    for i in 0..nloop {
        sum += foo_with_config(i, cfg);
    }
    sum
}

pub fn bench_apis(c: &mut Criterion) {
    c.bench_function("raw api", |b| b.iter(|| call_foo(black_box(10000))));
}

pub fn bench_apis_with_ps_and_raw_btree(c: &mut Criterion) {
    c.bench_function("raw api with ps and raw btree", |b| {
        b.iter(|| call_foo_with_ps_and_raw_btree(black_box(10000)))
    });
}

pub fn bench_apis_with_ps_optimized(c: &mut Criterion) {
    c.bench_function("raw api with ps optimized", |b| {
        b.iter(|| call_foo_with_ps_optimized(black_box(10000)))
    });
}

pub fn bench_apis_with_ps(c: &mut Criterion) {
    c.bench_function("raw api with ps", |b| {
        b.iter(|| call_foo_with_ps(black_box(10000)))
    });
}

pub fn bench_config_rs(c: &mut Criterion) {
    let cfg = config::Config::builder()
        .add_source(config::File::from_str(
            "{\"y\": 1}",
            config::FileFormat::Json,
        ))
        .build()
        .unwrap();
    c.bench_function("raw api with config-rs", |b| {
        b.iter(|| call_foo_with_config_rs(black_box(10000), &cfg))
    });
}

criterion_group!(
    benches,
    bench_apis,
    bench_apis_with_ps_and_raw_btree,
    bench_apis_with_ps_optimized,
    bench_apis_with_ps,
    bench_config_rs,
);
criterion_main!(benches);
