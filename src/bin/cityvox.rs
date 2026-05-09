//! cityvox — voxelize a generated city tile and render it in the engine.
//!
//! Demo bin: builds a `CityTile` from the procedural generator, rasterizes
//! buildings/roads/places into voxel blocks, opens a `VoxelWorld` and renders
//! it with the same camera/material setup as `sandbox`.

use std::path::PathBuf;

use bevy::input::mouse::MouseMotion;
use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PresentMode, PrimaryWindow};

use unvoga::blockstate;
use unvoga::core::city::{
    generate_demo_tile, voxelize_tile, BlockKind, CityTileId, DemoCityConfig, GeoPoint,
    VoxelizeConfig,
};
use unvoga::core::util::textureregistry as texreg;
use unvoga::core::voxel::block::Block;
use unvoga::core::voxel::blocks::{self, Id};
use unvoga::core::voxel::procgen::worldgenerator::FlatWorldGenerator;
use unvoga::core::voxel::rendering::voxelmaterial::VoxelMaterial;
use unvoga::core::voxel::world::{RenderChunkMarker, VoxelWorld};
use unvoga::game::cameras::{CameraContoller, CameraType};
use unvoga::prelude::SolidBlock;

const BLOCKS_DIR: &str = "./assets/debug/textures/blocks/";

macro_rules! reg_tex {
    ($name:ident) => {
        texreg::register(
            stringify!($name),
            PathBuf::from(BLOCKS_DIR).join(format!("{}.png", stringify!($name))),
        )
    };
}

fn main() {
    if std::env::var_os("BEVY_ASSET_ROOT").is_none()
        && std::env::var_os("CARGO_MANIFEST_DIR").is_none()
    {
        // SAFETY: process startup, single-threaded.
        unsafe { std::env::set_var("BEVY_ASSET_ROOT", env!("CARGO_MANIFEST_DIR")); }
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Unvoga: cityvox".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MaterialPlugin::<VoxelMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, fly_camera)
        .add_systems(PostUpdate, drive_world)
        .insert_resource(ClearColor(Color::srgb(0.45, 0.55, 0.75)))
        .run();
}

#[derive(Component)]
struct CameraMarker;

#[derive(Resource)]
struct CamRes {
    controller: CameraContoller,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    reg_tex!(dirt);
    reg_tex!(stone);
    reg_tex!(sand);
    reg_tex!(marble_01);
    reg_tex!(marble_02);
    reg_tex!(metal_grid);
    reg_tex!(fancy_wood);
    reg_tex!(fancy_wood_red);
    reg_tex!(fancy_wood_blue);

    blocks::register_block(SolidBlock::single("dirt", blockstate!(dirt), texreg::get_texture_index("dirt")));
    blocks::register_block(SolidBlock::single("stone", blockstate!(stone), texreg::get_texture_index("stone")));
    blocks::register_block(SolidBlock::single("sand", blockstate!(sand), texreg::get_texture_index("sand")));
    blocks::register_block(SolidBlock::single("marble_01", blockstate!(marble_01), texreg::get_texture_index("marble_01")));
    blocks::register_block(SolidBlock::single("marble_02", blockstate!(marble_02), texreg::get_texture_index("marble_02")));
    blocks::register_block(SolidBlock::single("metal_grid", blockstate!(metal_grid), texreg::get_texture_index("metal_grid")));
    blocks::register_block(SolidBlock::single("fancy_wood", blockstate!(fancy_wood), texreg::get_texture_index("fancy_wood")));
    blocks::register_block(SolidBlock::single("fancy_wood_red", blockstate!(fancy_wood_red), texreg::get_texture_index("fancy_wood_red")));
    blocks::register_block(SolidBlock::single("fancy_wood_blue", blockstate!(fancy_wood_blue), texreg::get_texture_index("fancy_wood_blue")));

    let texture_array = images.add(
        texreg::build_texture_array(256, 256).expect("build texture array"),
    );

    // Tile + voxelize.
    let tile = generate_demo_tile(
        CityTileId::new(0, 0, 0),
        GeoPoint::new(25.7617, -80.1918),
        DemoCityConfig::default(),
    );
    let cfg = VoxelizeConfig::default();
    let center_vox = (tile.tile_size_m * cfg.vox_per_meter * 0.5) as i32;

    // Ground generator fills the bedrock; voxelize_tile paints on top.
    let stone = blockstate!(stone).register();
    let dirt = blockstate!(dirt).register();
    let generator: Box<dyn unvoga::core::voxel::procgen::worldgenerator::WorldGenerator> =
        Box::new(FlatWorldGenerator::from_iter([(396u16, stone), (3, dirt)]));
    let mut world = VoxelWorld::open(
        "ignore/cityvox",
        4,
        (center_vox, 0, center_vox),
        texture_array,
        &mut commands,
        &mut meshes,
        &mut materials,
        Some(generator),
    );

    let road_id = blockstate!(stone).register();
    let bldg_a = blockstate!(marble_01).register();
    let bldg_b = blockstate!(marble_02).register();
    let bldg_c = blockstate!(fancy_wood_blue).register();
    let poi_id = blockstate!(metal_grid).register();
    let dirt_id = dirt;

    let bldg_palette = [bldg_a, bldg_b, bldg_c];
    let mut bldg_pick = 0usize;
    voxelize_tile(&tile, &cfg, |x, y, z, kind| {
        let id = match kind {
            BlockKind::Ground => dirt_id,
            BlockKind::Road => road_id,
            BlockKind::Building => {
                // Cycle palette per visited column for visual variety.
                let pick = bldg_palette[bldg_pick % bldg_palette.len()];
                if y % 4 == 0 { bldg_pick = bldg_pick.wrapping_add(1); }
                pick
            }
            BlockKind::Poi => poi_id,
        };
        // Lift everything one block above world y=0 so dirt bedrock is visible.
        world.set_block((x, y + 1, z), id);
    });

    // Camera: offset back + up looking at city center.
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            far: 2000.0,
            near: 0.05,
        }),
        Msaa::Off,
        Transform::from_xyz(center_vox as f32 - 60.0, 50.0, center_vox as f32 + 90.0)
            .looking_at(vec3(center_vox as f32, 8.0, center_vox as f32), Vec3::Y),
        CameraMarker,
    ));

    commands.insert_resource(world);
    commands.insert_resource(CamRes {
        controller: CameraContoller::new(CameraType::Free, vec2(0.0, 0.0), 18.0, 0.05),
    });

    if let Ok(mut primary) = window.single_mut() {
        primary.cursor_options.grab_mode = CursorGrabMode::Confined;
        primary.cursor_options.visible = false;
    }
}

fn fly_camera(
    mut motion: EventReader<MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<CameraMarker>>,
    mut cam: ResMut<CamRes>,
    mut exit: ResMut<Events<bevy::app::AppExit>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(bevy::app::AppExit::Success);
        return;
    }
    let Ok(transform) = camera.single_mut() else { return; };
    let dt = time.delta_secs();
    let look: Vec2 = motion.read().map(|e| e.delta).sum();
    let mut ctl = cam.controller.begin_transform(transform);
    ctl.rotate(look, dt, 1.0);
    let mut t = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { t += Vec3::NEG_Z; }
    if keys.pressed(KeyCode::KeyS) { t += Vec3::Z; }
    if keys.pressed(KeyCode::KeyA) { t += Vec3::NEG_X; }
    if keys.pressed(KeyCode::KeyD) { t += Vec3::X; }
    if keys.pressed(KeyCode::Space) { t += Vec3::Y; }
    if keys.pressed(KeyCode::ControlLeft) { t += Vec3::NEG_Y; }
    let mult = if keys.pressed(KeyCode::ShiftLeft) { 4.0 } else { 1.0 };
    ctl.translate(t, dt, mult);
}

fn drive_world(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<VoxelMaterial>>,
    storage_buffers: ResMut<Assets<bevy::render::storage::ShaderStorageBuffer>>,
    render_chunks: Query<&mut Transform, With<RenderChunkMarker>>,
    mut world: ResMut<VoxelWorld>,
) {
    world.talk_to_bevy(commands, meshes, materials, storage_buffers, render_chunks);
}
