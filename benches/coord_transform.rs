// Coordinate transforms. Called inside occluded_by per-pixel — multiplies
// occlusion cost. Driver for table-lookup vs runtime-compute optimizations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use unvoga::core::voxel::direction::Direction;
use unvoga::core::voxel::occlusionshape::OcclusionRect;
use unvoga::prelude::{Flip, Orientation, Rotation};

fn bench_source_face_coord(c: &mut Criterion) {
    let mut g = c.benchmark_group("source_face_coord");
    for &roti in &[0u8, 5, 13, 23] {
        let o = Orientation::new(Rotation(roti), Flip(0b101));
        g.bench_with_input(BenchmarkId::from_parameter(roti), &o, |b, &o| {
            b.iter(|| {
                black_box(o).source_face_coord(
                    black_box(Direction::PosY),
                    black_box((3i8, 5i8)),
                )
            })
        });
    }
    g.finish();
}

fn bench_map_face_coord(c: &mut Criterion) {
    let o = Orientation::new(Rotation(7), Flip(0b011));
    c.bench_function("map_face_coord", |b| {
        b.iter(|| {
            black_box(o).map_face_coord(
                black_box(Direction::PosX),
                black_box((3i8, 5i8)),
            )
        })
    });
}

fn bench_rect_transform_face(c: &mut Criterion) {
    let rect = OcclusionRect::new(2, 3, 8, 8);
    let o = Orientation::new(Rotation(7), Flip(0b011));
    c.bench_function("OcclusionRect::transform_face", |b| {
        b.iter(|| black_box(rect).transform_face(black_box(o), black_box(Direction::PosY)))
    });
}

fn bench_rect_rotate(c: &mut Criterion) {
    let rect = OcclusionRect::new(2, 3, 8, 8);
    c.bench_function("OcclusionRect::rotate", |b| {
        b.iter(|| black_box(rect).rotate(black_box(2)))
    });
}

fn bench_rect_downsample(c: &mut Criterion) {
    let rect = OcclusionRect::new(0, 0, 16, 16);
    let mut g = c.benchmark_group("OcclusionRect::downsample");
    for &n in &[2u8, 4, 8] {
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| black_box(rect).downsample(black_box(n)))
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_source_face_coord,
    bench_map_face_coord,
    bench_rect_transform_face,
    bench_rect_rotate,
    bench_rect_downsample,
);
criterion_main!(benches);
