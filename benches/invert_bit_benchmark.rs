#![allow(unused)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unvoga::core::math::bit::*;

fn invert_bit_old<I: ShiftIndex>(value: u64, index: I) -> u64 {
    let bit = value.get_bit(index);
    value.set_bit(index, !bit)
}

fn invert_bit_new<I: ShiftIndex>(value: u64, index: I) -> u64 {
    value.invert_bit(index)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("invert_bit_old", |b| b.iter(|| invert_bit_old(black_box(1u64), black_box(0))));
    c.bench_function("invert_bit_new", |b| b.iter(|| invert_bit_new(black_box(1u64), black_box(0))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);