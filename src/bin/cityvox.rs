//! cityvox — voxelize a generated city tile and render it in the engine.
//!
//! Demo bin: builds a `CityTile` from the procedural generator, rasterizes
//! buildings/roads/places into voxel blocks, opens a `VoxelWorld` and renders
//! it with a fly camera.

use std::path::PathBuf;

use bevy::input::mouse::MouseMotion;
use bevy::math::{vec2, vec3};
use bevy::pbr::{AmbientLight, DirectionalLight, DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PresentMode, PrimaryWindow};

use unvoga::blockstate;
use unvoga::core::city::{
    generate_demo_tile, voxelize_tile, BlockKind, CityTileId, DemoCityConfig, GeoPoint,
    VoxelizeConfig,
};
use unvoga::core::util::textureregistry as texreg;
use unvoga::core::voxel::blocks::{self, Id};
use unvoga::core::voxel::procgen::worldgenerator::FlatWorldGenerator;
use unvoga::core::voxel::rendering::voxelmaterial::VoxelMaterial;
use unvoga::core::voxel::world::{RenderChunkMarker, VoxelWorld};
use unvoga::game::cameras::{CameraContoller, CameraType};
use unvoga::prelude::SolidBlock;

const BLOCKS_DIR: &str = "./assets/debug/textures/blocks/";
const RENDER_DISTANCE: u8 = 6;

macro_rules! reg_tex {
    ($name:ident) => {
        texreg::register(
            stringify!($name),
            PathBuf::from(BLOCKS_DIR).join(format!("{}.png", stringify!($name))),
        )
    };
}

#[derive(Resource)]
struct CityPalette {
    ground: Id,
    sand: Id,
    road: Id,
    sidewalk: Id,
    roof: Id,
    poi: Id,
    building: [Id; 4],
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
        .insert_resource(AmbientLight {
            color: Color::srgb(0.85, 0.88, 0.95),
            brightness: 250.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb(0.55, 0.68, 0.85)))
        .add_systems(Startup, setup)
        .add_systems(Update, fly_camera)
        .add_systems(PostUpdate, drive_world)
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
    reg_tex!(stone_bricks);
    reg_tex!(cement);
    reg_tex!(marble_01);
    reg_tex!(marble_02);
    reg_tex!(metal_grid);
    reg_tex!(fancy_wood);
    reg_tex!(fancy_wood_red);
    reg_tex!(fancy_wood_blue);

    blocks::register_block(SolidBlock::single("dirt", blockstate!(dirt), texreg::get_texture_index("dirt")));
    blocks::register_block(SolidBlock::single("stone", blockstate!(stone), texreg::get_texture_index("stone")));
    blocks::register_block(SolidBlock::single("sand", blockstate!(sand), texreg::get_texture_index("sand")));
    blocks::register_block(SolidBlock::single("cement", blockstate!(cement), texreg::get_texture_index("cement")));
    blocks::register_block(SolidBlock::vertical_block(
        "stone_bricks",
        blockstate!(stone_bricks),
        texreg::get_texture_index("cement"),
        texreg::get_texture_index("stone_bricks"),
    ));
    blocks::register_block(SolidBlock::single("marble_01", blockstate!(marble_01), texreg::get_texture_index("marble_01")));
    blocks::register_block(SolidBlock::single("marble_02", blockstate!(marble_02), texreg::get_texture_index("marble_02")));
    blocks::register_block(SolidBlock::single("metal_grid", blockstate!(metal_grid), texreg::get_texture_index("metal_grid")));
    blocks::register_block(SolidBlock::single("fancy_wood", blockstate!(fancy_wood), texreg::get_texture_index("fancy_wood")));
    blocks::register_block(SolidBlock::single("fancy_wood_red", blockstate!(fancy_wood_red), texreg::get_texture_index("fancy_wood_red")));
    blocks::register_block(SolidBlock::single("fancy_wood_blue", blockstate!(fancy_wood_blue), texreg::get_texture_index("fancy_wood_blue")));

    let texture_array = images.add(
        texreg::build_texture_array(256, 256).expect("build texture array"),
    );

    let palette = CityPalette {
        ground: blockstate!(dirt).register(),
        sand: blockstate!(sand).register(),
        road: blockstate!(stone).register(),
        sidewalk: blockstate!(cement).register(),
        roof: blockstate!(stone_bricks).register(),
        poi: blockstate!(metal_grid).register(),
        building: [
            blockstate!(marble_01).register(),
            blockstate!(marble_02).register(),
            blockstate!(fancy_wood_blue).register(),
            blockstate!(fancy_wood_red).register(),
        ],
    };

    // Tile + voxelize. Smaller tile + higher vox/m = same world span, more detail.
    let demo_cfg = DemoCityConfig {
        tile_size_m: 256.0,
        block_size_m: 32.0,
        ..DemoCityConfig::default()
    };
    let vox_cfg = VoxelizeConfig {
        vox_per_meter: 0.5,
        building_palette_size: palette.building.len() as u8,
        ..VoxelizeConfig::default()
    };
    let tile = generate_demo_tile(
        CityTileId::new(0, 0, 0),
        GeoPoint::new(25.7617, -80.1918),
        demo_cfg,
    );
    let center_vox = (tile.tile_size_m * vox_cfg.vox_per_meter * 0.5) as i32;

    let stone_id = blockstate!(stone).register();
    let dirt_id = blockstate!(dirt).register();
    let generator: Box<dyn unvoga::core::voxel::procgen::worldgenerator::WorldGenerator> =
        Box::new(FlatWorldGenerator::from_iter([(396u16, stone_id), (3, dirt_id)]));
    let mut world = VoxelWorld::open(
        "ignore/cityvox",
        RENDER_DISTANCE,
        (center_vox, 0, center_vox),
        texture_array,
        &mut commands,
        &mut meshes,
        &mut materials,
        Some(generator),
    );

    voxelize_tile(&tile, &vox_cfg, |x, y, z, kind| {
        let id = match kind {
            BlockKind::Ground => palette.ground,
            BlockKind::Sand => palette.sand,
            BlockKind::Road => palette.road,
            BlockKind::Sidewalk => palette.sidewalk,
            BlockKind::Building { palette: p } => palette.building[p as usize % palette.building.len()],
            BlockKind::Roof => palette.roof,
            BlockKind::Poi => palette.poi,
        };
        // Lift one block above world y=0 so the dirt bedrock is visible.
        world.set_block((x, y + 1, z), id);
    });

    // Sun.
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.96, 0.86),
            illuminance: 12_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(vec3(0.4, -1.0, 0.25), Vec3::Y),
    ));

    // Camera with MSAA + fog.
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            far: 2000.0,
            near: 0.05,
        }),
        Msaa::Sample4,
        DistanceFog {
            color: Color::srgb(0.55, 0.68, 0.85),
            falloff: FogFalloff::Linear { start: 80.0, end: 220.0 },
            ..default()
        },
        Transform::from_xyz(center_vox as f32 - 60.0, 50.0, center_vox as f32 + 90.0)
            .looking_at(vec3(center_vox as f32, 8.0, center_vox as f32), Vec3::Y),
        CameraMarker,
    ));

    commands.insert_resource(world);
    commands.insert_resource(palette);
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
