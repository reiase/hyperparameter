use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hyperparameter::*;

fn foo(x: i64, y: i64) -> i64 {
    x + y
}

fn foo_with_ps(x: i64) -> i64 {
    with_params! {
        get y = y or 0;

        x+y
    }
}

fn call_foo(nloop: i64) -> i64 {
    let mut sum = 0;
    for i in 0..nloop {
        sum = sum + foo(black_box(i), black_box(42));
    }
    sum
}

fn call_foo_with_ps(nloop: i64) -> i64 {
    let mut sum = 0;
    for i in 0..nloop {
        with_params! {
            set y = 42;

            sum = sum + foo_with_ps(black_box(i));
        }
    }
    sum
}

fn call_foo_with_ps_optimized(nloop: i64) -> i64 {
    let mut sum = 0;

    with_params! {
        set y = 42;

        for i in 0..nloop {
            sum = sum + foo_with_ps(black_box(i));
        }
    }
    sum
}

pub fn bench_apis(c: &mut Criterion) {
    c.bench_function("raw api", |b| b.iter(|| call_foo(black_box(100000))));
}

pub fn bench_apis_with_ps_optimized(c: &mut Criterion) {
    c.bench_function("raw api with ps optimized", |b| {
        b.iter(|| call_foo_with_ps_optimized(black_box(100000)))
    });
}

pub fn bench_apis_with_ps(c: &mut Criterion) {
    c.bench_function("raw api with ps", |b| {
        b.iter(|| call_foo_with_ps(black_box(100000)))
    });
}

criterion_group!(
    benches,
    bench_apis,
    bench_apis_with_ps_optimized,
    bench_apis_with_ps,
);
criterion_main!(benches);
