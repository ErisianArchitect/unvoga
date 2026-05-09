// Chunk-scale occlusion workload. Simulates the cost of testing every
// face of every solid block against its 6 neighbors, which is what
// meshing has to do per-chunk.
//
// Use as a coarse regression detector: if this gets faster, chunk
// remesh will too. Tune one bench above (occluded_by) and watch this
// move in the same direction.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unvoga::core::voxel::direction::Direction;
use unvoga::core::voxel::occluder::Occluder;
use unvoga::core::voxel::occlusionshape::{
    OcclusionRect, OcclusionShape, OcclusionShape16x16, OcclusionShape8x8,
};
use unvoga::prelude::Orientation;

const SIDE: usize = 16; // 16^3 = 4096 voxels (one section)

fn build_grid(mode: GridMode) -> Vec<Occluder> {
    (0..SIDE * SIDE * SIDE)
        .map(|i| {
            let shape = match mode {
                GridMode::AllFull => OcclusionShape::Full,
                GridMode::Mixed => match i % 4 {
                    0 => OcclusionShape::Full,
                    1 => OcclusionShape::Empty,
                    2 => OcclusionShape::S8x8(OcclusionShape8x8::new(u64::MAX)),
                    _ => OcclusionShape::Rect(OcclusionRect::new(0, 0, 16, 16)),
                },
                GridMode::Detailed => OcclusionShape::S16x16(OcclusionShape16x16::new([u16::MAX; 16])),
            };
            Occluder::new(
                shape.clone(),
                shape.clone(),
                shape.clone(),
                shape.clone(),
                shape.clone(),
                shape,
            )
        })
        .collect()
}

#[derive(Copy, Clone)]
enum GridMode {
    AllFull,
    Mixed,
    Detailed,
}

fn idx(x: usize, y: usize, z: usize) -> usize {
    (y * SIDE + z) * SIDE + x
}

fn count_visible_faces(grid: &[Occluder]) -> u32 {
    let o = Orientation::default();
    let mut visible = 0u32;
    for y in 0..SIDE {
        for z in 0..SIDE {
            for x in 0..SIDE {
                let me = &grid[idx(x, y, z)];
                let neighbors = [
                    (x.wrapping_sub(1), y, z, Direction::NegX),
                    (x + 1, y, z, Direction::PosX),
                    (x, y.wrapping_sub(1), z, Direction::NegY),
                    (x, y + 1, z, Direction::PosY),
                    (x, y, z.wrapping_sub(1), Direction::NegZ),
                    (x, y, z + 1, Direction::PosZ),
                ];
                for (nx, ny, nz, face) in neighbors {
                    if nx >= SIDE || ny >= SIDE || nz >= SIDE {
                        visible += 1;
                        continue;
                    }
                    let other = &grid[idx(nx, ny, nz)];
                    if !me.occluded_by(o, face, other, o) {
                        visible += 1;
                    }
                }
            }
        }
    }
    visible
}

fn bench_chunk_occlusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_occlusion_16cubed");
    group.sample_size(20); // each iteration scans 24576 face-pairs

    let all_full = build_grid(GridMode::AllFull);
    let mixed = build_grid(GridMode::Mixed);
    let detailed = build_grid(GridMode::Detailed);

    group.bench_function("all_full", |b| {
        b.iter(|| count_visible_faces(black_box(&all_full)))
    });
    group.bench_function("mixed", |b| {
        b.iter(|| count_visible_faces(black_box(&mixed)))
    });
    group.bench_function("all_detailed_16x16", |b| {
        b.iter(|| count_visible_faces(black_box(&detailed)))
    });

    group.finish();
}

criterion_group!(benches, bench_chunk_occlusion);
criterion_main!(benches);
