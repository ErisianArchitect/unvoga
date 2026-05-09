// Bit-level primitives. Hot in occlusion + meshing inner loops.
// Targets to optimize: move_bits (permutations), set_bitmask (storage packing).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use unvoga::core::math::bit::*;

fn bench_invert_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("invert_bit");
    for &index in &[0u32, 7, 31, 63] {
        group.bench_with_input(BenchmarkId::from_parameter(index), &index, |b, &i| {
            b.iter(|| black_box(0xDEADBEEF_u64).invert_bit(black_box(i)))
        });
    }
    group.finish();
}

fn bench_set_bit(c: &mut Criterion) {
    c.bench_function("set_bit/u64", |b| {
        b.iter(|| black_box(0u64).set_bit(black_box(31u32), black_box(true)))
    });
}

fn bench_get_bit(c: &mut Criterion) {
    c.bench_function("get_bit/u64", |b| {
        b.iter(|| black_box(0xDEADBEEF_u64).get_bit(black_box(31u32)))
    });
}

fn bench_get_bitmask(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_bitmask/u64");
    for &len in &[4u32, 8, 16, 32] {
        group.bench_with_input(BenchmarkId::from_parameter(len), &len, |b, &n| {
            b.iter(|| black_box(0xDEADBEEFCAFEBABE_u64).get_bitmask(black_box(0..n)))
        });
    }
    group.finish();
}

fn bench_set_bitmask(c: &mut Criterion) {
    c.bench_function("set_bitmask/u64/16bit", |b| {
        b.iter(|| {
            black_box(0u64).set_bitmask(black_box(8..24), black_box(0xFFFF))
        })
    });
}

fn bench_move_bits(c: &mut Criterion) {
    // Identity permutation, 64 bits — exercises the permutation correctness
    // fix (snapshot before write) and shows worst-case cost.
    let indices: Vec<u32> = (0..64).collect();
    c.bench_function("move_bits/u64/identity", |b| {
        b.iter(|| black_box(0xDEADBEEFCAFEBABE_u64).move_bits(black_box(indices.iter().copied())))
    });

    // Reverse permutation — meaningful permutation, full bit-shuffle.
    let rev: Vec<u32> = (0..64).rev().collect();
    c.bench_function("move_bits/u64/reverse", |b| {
        b.iter(|| black_box(0xDEADBEEFCAFEBABE_u64).move_bits(black_box(rev.iter().copied())))
    });
}

criterion_group!(
    benches,
    bench_invert_bit,
    bench_set_bit,
    bench_get_bit,
    bench_get_bitmask,
    bench_set_bitmask,
    bench_move_bits,
);
criterion_main!(benches);
