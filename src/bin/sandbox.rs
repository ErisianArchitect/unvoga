

fn main() {
    println!("World Test");
    let mut world = VoxelWorld::new(32, Coord::new(0, -10000, 0));
    blocks::register_block(DirtBlock);
    blocks::register_block(RotatedBlock);
    blocks::register_block(DebugBlock);
    println!(" World Bounds: {:?}", world.bounds());
    println!("Render Bounds: {:?}", world.render_bounds());
    println!("  Block Count: {}", world.bounds().volume());
    let air = StateRef::AIR;
    let debug = blockstate!(debug).register();
    let debug_data = blockstate!(debug, withdata = true).register();
    let dirt = blockstate!(dirt).register();
    let rot1 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 1)).register();
    let rot2 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 3)).register();
    
    itertools::iproduct!(15..16, 0..1, 15..16).for_each(|(y, z, x)| {
        world.set((x, y, z), debug_data);
    });
    world.set((15, 15, 15), debug_data);

    itertools::iproduct!(0..16, 0..16, 0..16).for_each(|(y, z, x)| {
        world.set((x, y, z), air);
    });
    itertools::iproduct!(0..16, 0..16, 0..16).for_each(|(y, z, x)| {
        let faces = world.occlusion((x, y, z));
        if faces != Occlusion::UNOCCLUDED {
            println!("Occluded at ({x:2}, {y:2}, {z:2})");
        }
    });
    let usage = world.dynamic_usage();
    println!("Memory: {usage}");
}