use std::path::Path;

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
    let top = miny + 240;
    let y_trans = |t: f64| {
        (t.clamp(0.0, 1.0) * 240.0) as i32
    };
    let dirt_block = blockstate!(dirt).register();
    let stone_block = blockstate!(stone).register();
    for z in minz..maxz {
        for x in minx..maxx {
            let point = Point::new(x as f64, z as f64);
            // let mesa = mesas.sample(point);
            let mountain = mountains.sample(point);
            // let stone = stones.sample(point);
            // let stone_height = (stone * 10.0) as i32;
            // let height = y_trans(mesa * mountain);
            let height = y_trans(mountain);
            for y in miny..miny + height {
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