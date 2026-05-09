// Occlusion query — runs once per face per neighbor pair during chunk meshing.
// 16^3 chunk * 6 faces * neighbor query = ~24K calls per chunk per remesh.
// Target perf: sub-100ns per call for the common case.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use unvoga::core::voxel::direction::Direction;
use unvoga::core::voxel::occluder::Occluder;
use unvoga::core::voxel::occlusionshape::{
    OcclusionRect, OcclusionShape, OcclusionShape16x16, OcclusionShape2x2, OcclusionShape4x4,
    OcclusionShape8x8,
};
use unvoga::prelude::Orientation;

fn occluder_from(shape: OcclusionShape) -> Occluder {
    Occluder::new(
        shape.clone(),
        shape.clone(),
        shape.clone(),
        shape.clone(),
        shape.clone(),
        shape,
    )
}

fn bench_occluded_by_full_full(c: &mut Criterion) {
    let a = Occluder::FULL_FACES;
    let b = Occluder::FULL_FACES;
    let o = Orientation::default();
    c.bench_function("occluded_by/full_vs_full", |bn| {
        bn.iter(|| {
            black_box(&a).occluded_by(
                black_box(o),
                black_box(Direction::PosY),
                black_box(&b),
                black_box(o),
            )
        })
    });
}

fn bench_occluded_by_full_empty(c: &mut Criterion) {
    let a = Occluder::FULL_FACES;
    let b = Occluder::EMPTY_FACES;
    let o = Orientation::default();
    c.bench_function("occluded_by/full_vs_empty", |bn| {
        bn.iter(|| {
            black_box(&a).occluded_by(
                black_box(o),
                black_box(Direction::PosY),
                black_box(&b),
                black_box(o),
            )
        })
    });
}

fn bench_occluded_by_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("occluded_by/grid");

    let s2 = occluder_from(OcclusionShape::S2x2(OcclusionShape2x2::new(0b1111)));
    let s4 = occluder_from(OcclusionShape::S4x4(OcclusionShape4x4::new(0xFFFF)));
    let s8 = occluder_from(OcclusionShape::S8x8(OcclusionShape8x8::new(u64::MAX)));
    let s16 = occluder_from(OcclusionShape::S16x16(OcclusionShape16x16::new([u16::MAX; 16])));
    let rect = occluder_from(OcclusionShape::Rect(OcclusionRect::new(0, 0, 16, 16)));
    let o = Orientation::default();

    for (label, a, b) in [
        ("s2x2_vs_s2x2", &s2, &s2),
        ("s4x4_vs_s4x4", &s4, &s4),
        ("s8x8_vs_s8x8", &s8, &s8),
        ("s16x16_vs_s16x16", &s16, &s16),
        ("rect_vs_s8x8", &rect, &s8),
        ("rect_vs_s16x16", &rect, &s16),
        ("s16x16_vs_s8x8", &s16, &s8),
        ("s16x16_vs_rect", &s16, &rect),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &(a, b), |bn, &(a, b)| {
            bn.iter(|| {
                black_box(a).occluded_by(
                    black_box(o),
                    black_box(Direction::PosY),
                    black_box(b),
                    black_box(o),
                )
            })
        });
    }
    group.finish();
}

fn bench_shape_get(c: &mut Criterion) {
    let s16 = OcclusionShape16x16::new([u16::MAX; 16]);
    let s8 = OcclusionShape8x8::new(u64::MAX);
    let s4 = OcclusionShape4x4::new(0xFFFF);
    let s2 = OcclusionShape2x2::new(0b1111);
    let mut g = c.benchmark_group("shape_get");
    g.bench_function("16x16", |b| b.iter(|| black_box(&s16).get(black_box(7), black_box(7))));
    g.bench_function("8x8", |b| b.iter(|| black_box(s8).get(black_box(3), black_box(3))));
    g.bench_function("4x4", |b| b.iter(|| black_box(s4).get(black_box(2), black_box(2))));
    g.bench_function("2x2", |b| b.iter(|| black_box(s2).get(black_box(1), black_box(1))));
    g.finish();
}

criterion_group!(
    benches,
    bench_occluded_by_full_full,
    bench_occluded_by_full_empty,
    bench_occluded_by_grid,
    bench_shape_get,
);
criterion_main!(benches);
