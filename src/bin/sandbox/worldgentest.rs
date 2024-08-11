use std::path::Path;

use unvoga::core::math::grid::calculate_center_offset;
use unvoga::core::voxel::world::WORLD_BOTTOM;
use unvoga::prelude::Coord;
use unvoga::{blockstate, core::voxel::world::VoxelWorld};
use unvoga::core::voxel::procgen::noise::*;


pub fn import_generator<P: AsRef<Path>, B: AsRef<[u8]>>(path: P, seed: B) -> NoiseGen {
    NoiseGen::from_config(NoiseGenConfig::import(path).expect("Failed to import"), seed)
}

pub fn generate_world(world: &mut VoxelWorld) {
    // let mesas = import_generator("./assets/debug/generators/test.simp", "mesas");
    let mountains = import_generator("./assets/debug/generators/mountains.simp", "mountains");
    // let stones = import_generator("./assets/debug/generators/stone.simp", "stone");
    // let config = worldgen::noise::NoiseGenConfig::import("./assets/debug/generators/mesas.simp").expect("Failed to import config");
    // let generator = worldgen::noise::NoiseGen::from_config(config, "idk");
    let (minx, miny, minz) = world.render_bounds().min;
    let (maxx, maxy, maxz) = world.render_bounds().max;
    let center_x = (maxx - minx) / 2 + minx;
    let center_y = (maxy - miny) / 2 + miny;
    let center_z = (maxz - minz) / 2 + minz;

    let center_offset = calculate_center_offset(3, Coord::new(center_x, center_y, center_z), Some(VoxelWorld::WORLD_BOUNDS)).xz();
    let width_depth = 16 * 6;
    let top = WORLD_BOTTOM + 240;
    let y_trans = |t: f64| {
        (t.clamp(0.0, 1.0) * 240.0) as i32
    };
    let dirt_block = blockstate!(dirt).register();
    let stone_block = blockstate!(stone).register();
    for z in center_offset.1..center_offset.1 + width_depth {
        for x in center_offset.0..center_offset.0 + width_depth {
            let point = Point::new(x as f64, z as f64);
            // let mesa = mesas.sample(point);
            let mountain = mountains.sample(point);
            // let stone = stones.sample(point);
            // let stone_height = (stone * 10.0) as i32;
            // let height = y_trans(mesa * mountain);
            let height = y_trans(mountain);
            for y in WORLD_BOTTOM..WORLD_BOTTOM + height {
                world.set_block((x, y, z), stone_block);
            }
            // let y_top = miny + height;
            // for y in miny..y_top-stone_height {
            //     world.set_block((x, y, z), stone_block);
            // }
            // for y in y_top-stone_height..y_top {
            //     world.set_block((x, y, z), dirt_block);
            // }
        }
    }
}

fn fda_angle(fda: (f64,f64)) -> f64 {
    (fda.1 / fda.0).atan()
}

fn fda_magnitude(fda: (f64, f64)) -> f64 {
    (fda.0*fda.0 + fda.1*fda.1).sqrt()
}

fn finite_difference_approximation<F: Fn(f64, f64) -> f64>(noise_fn: F, x: f64, y: f64, epsilon: f64) -> (f64, f64) {
    let e2 = epsilon * 2.;
    let diff_x = (noise_fn(x + epsilon, y) - noise_fn(x - epsilon, y)) / e2;
    let diff_y = (noise_fn(x, y + epsilon) - noise_fn(x, y - epsilon)) / e2;
    (diff_x, diff_y)
}