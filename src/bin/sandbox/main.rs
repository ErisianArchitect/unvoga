#![allow(unused)]
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Instant;
use std::{sync::Arc, thread, time::Duration};

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::app::DynEq;
use bevy::math::{vec2, vec3, IVec2};
use bevy::window::{CursorGrabMode, PresentMode, PrimaryWindow};
use bevy_egui::EguiPlugin;
use hashbrown::HashMap;
use rollgrid::rollgrid3d::Bounds3D;
use unvoga::core::voxel::rendering::voxelmaterial::VoxelMaterial;
use unvoga::core::voxel::rendering::voxelmesh::MeshData;
use unvoga::core::voxel::world::{RaycastResult, RenderChunkMarker};
use unvoga::prelude::*;
use unvoga::core::voxel::region::regionfile::RegionFile;
use unvoga::prelude::*;
use unvoga::core::error::*;
use unvoga::{blockstate, core::{util::counter::AtomicCounter, voxel::{block::Block, blocks::{self, Id}, coord::Coord, direction::Direction, faces::Faces, occluder::Occluder, occlusionshape::{OcclusionShape, OcclusionShape16x16, OcclusionShape2x2}, tag::Tag, world::{query::Enabled, PlaceContext, VoxelWorld}}}};

#[derive(Debug, Default)]
struct BlockRegistry {

}

impl BlockRegistry {
    pub fn new() -> Self {
        Self {

        }
    }
    
    pub fn foo(&self) {
        println!("Hello, world, from BlockRegistry.");
    }
}

static BLOCKS: LazyLock<BlockRegistry> = LazyLock::new(BlockRegistry::new);

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(bevy::window::Window {
                title: "Unvoga".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(MaterialPlugin::<VoxelMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (update_input, update).chain())
        // .add_systems(Update, update_input.before(update))SSSSSSSnu.run_if(in_state(GameState::MainMenu)))
        // .add_systems(OnEnter(GameState::LoadingScreen), enter_loading_screen)
        // .add_systems(OnExit(GameState::LoadingScreen), cleanup_system::<cleanup::LoadingScreen>)
        // .add_systems(OnEnter(GameState::MainMenu), on_enter_main_menu)
        // .add_systems(OnExit(GameState::MainMenu), cleanup_system::<cleanup::Menu>)
        // .add_systems(OnEnter(GameState::SinglePlayer), on_enter_singleplayer)
        // .add_systems(OnExit(GameState::SinglePlayer), cleanup_system::<cleanup::SinglePlayer>)
        .insert_resource(Assets::<VoxelMaterial>::default())
        .insert_resource(Assets::<Mesh>::default())
        // .insert_resource(Assets::<Image>::default())
        .insert_resource(ClearColor(Color::rgb(0.2,0.2,0.2)))
        // .insert_resource(Msaa::Off)
        .run();
    // BLOCKS.foo();
    // return;
    // use unvoga::core::voxel::direction::Direction;

    // println!("World Test");
    // blocks::register_block(DirtBlock);
    // blocks::register_block(RotatedBlock);
    // blocks::register_block(DebugBlock);
    // let air = Id::AIR;
    // let debug = blockstate!(debug).register();
    // let debug_data = blockstate!(debug, withdata = true, flip=Flip::X | Flip::Y, orientation=Orientation::new(Rotation::new(Direction::NegZ, 3), Flip::X | Flip::Y)).register();
    // let enabled = blockstate!(debug, enabled = true).register();
    // let dirt = blockstate!(dirt).register();
    // let rot1 = blockstate!(rotated).register();
    // let rot2 = blockstate!(rotated, orientation=Orientation::new(Rotation::new(Direction::NegZ, 1), Flip::XYZ)).register();
    // // let mut world = VoxelWorld::open("ignore/test_world", 16, (0, 0, 0));

    // let coord1 = (1, 1, 1);
    // let coord2 = (1, 0, 1);
    // world.set_block(coord1, dirt);
    // world.set_block(coord2, dirt);
    // // world.set_block(coord1, rot2);
    // // world.set_block(coord2, rot1);
    // let occl1 = world.get_occlusion(coord1);
    // let occl2 = world.get_occlusion(coord2);
    // println!("{occl1}");
    // println!("{occl2}");
    // return;

    // let usage = world.dynamic_usage();
    // println!("     Memory Usage: {usage}");
    // println!("     World Bounds: {:?}", world.bounds());
    // println!("    Render Bounds: {:?}", world.render_bounds());
    // println!("      Block Count: {}", world.bounds().volume());
    // println!("World Block Count: {}", VoxelWorld::WORLD_BOUNDS.volume());

    // println!("Update after load.");
    // world.update();
    // println!("Getting block");
    // let coord = (2,3,4);
    // {
    //     let block = world.get_block(coord);
    //     let occ = world.get_occlusion(coord);
    //     let block_light = world.get_block_light(coord);
    //     let sky_light = world.get_sky_light(coord);
    //     let light_level = world.get_light_level(coord);
    //     let enabled = world.enabled(coord);
    //     let data = world.get_data(coord);
    //     println!("      Block: {block}");
    //     println!("  Occlusion: {occ}");
    //     println!("Block Light: {block_light}");
    //     println!("  Sky Light: {sky_light}");
    //     println!("Light Level: {light_level}");
    //     println!("    Enabled: {enabled}");
    //     println!("       Data: {data:?}");
    // }
    // // drop(data);
    // world.set_block(coord, debug);
    // world.set_data(coord, Tag::from("This data should be deleted."));
    // println!("Setting air.");
    // // world.set_block(coord, air);
    // if let Some(data) = world.get_data(coord) {
    //     println!("Data that shouldn't exist: {data:?}");
    // }
    // world.set_block_light(coord, 1);
    // world.set_sky_light(coord, 6);
    // world.set_enabled(coord, true);
    // world.update();
    // let height = world.height(2, 4);
    // println!("Height: {height}");
    // world.save_world();
    // return;
    // let tag = Tag::from(["test", "Hello, world"]);
    // println!("{tag:?}");
    // world.move_center((1024*1024, 0, 1024*1024));
    // println!("Update after move");
    // world.update();
    // println!("Update Queue Length: {}", world.update_queue.update_queue.len());
    // let coord = (1024*1024 + 3, 3, 1024*1024 + 3);
    // let block = world.get_block(coord);
    // println!("Far Block: {block}");
    // let height = world.height(coord.0, coord.2);
    // println!("Height: {height}");
    // let block = world.get_block(coord);
    // println!("Block: {block}");
    // world.set_block(coord, enabled);
    // let block = world.get_block(coord);
    // println!("Block: {block}");
    // world.save_world();

    // let coord = Coord::new(13,12,69).chunk_coord();
    // {
    //     let chunk = world.get_chunk((coord.x, coord.z)).unwrap();
    //     println!("Edit Time: {}", chunk.edit_time.0);
    // }
    // std::thread::sleep(Duration::from_secs(2));
    // itertools::iproduct!(0..2, 0..2).for_each(|(y, x)| {
    //     world.set_block((x, 0, y), enabled);
    // });
    // world.set_block((13,12, 69), debug_data);
    // world.call((13,12,69), "test", Tag::from("Hello, world"));
    // world.call((13,12,69), "set_enabled", true);
    // let (state, enabled): (Id, bool) = world.query::<_, (Id, Enabled)>((13,12,69));
    // println!("{state} {enabled}");
    // println!("Frame 1");
    // {
    //     let chunk = world.get_chunk((coord.x, coord.z)).unwrap();
    //     println!("Edit Time: {}", chunk.edit_time.0);
    // }
    // world.update();
    // world.call((13,12,69), "set_enabled", false);
    // println!("Frame 2");
    // world.update();
    

    // println!("Write/Read Test");
    // let now = std::time::Instant::now();
    // write_read_test().expect("Failure");
    // let elapsed = now.elapsed();
    // println!("Elapsed secs: {}", elapsed.as_secs_f64());
    
    // let usage = world.dynamic_usage();
    // println!("Memory: {usage}");
}

#[derive(Resource)]
struct VoxelWorldRes {
    world: VoxelWorld,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut giz_store: ResMut<GizmoConfigStore>,
) {
    for (_, config, _) in giz_store.iter_mut() {
        config.depth_bias = -1.0;
    }
    let side_texture_paths = vec![
        "./assets/debug/textures/cube_sides/pos_y.png",
        "./assets/debug/textures/cube_sides/pos_x.png",
        "./assets/debug/textures/cube_sides/pos_z.png",
        "./assets/debug/textures/cube_sides/neg_y.png",
        "./assets/debug/textures/cube_sides/neg_x.png",
        "./assets/debug/textures/cube_sides/neg_z.png",
    ];
    let cube_sides_texarray = images.add(unvoga::core::util::texture_array::create_texture_array_from_paths(512, 512, side_texture_paths).expect("Failed to create texture array."));
    let pos_y_mesh = MeshData {
        vertices: vec![
            vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, -0.5),
            vec3(-0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5),
        ],
        normals: vec![
            Vec3::Y, Vec3::Y,
            Vec3::Y, Vec3::Y,
        ],
        uvs: vec![
            vec2(0.0, 0.0), vec2(1.0, 0.0),
            vec2(0.0, 1.0), vec2(1.0, 1.0),
        ],
        texindices: vec![
            0, 0,
            0, 0,
        ],
        indices: vec![
            0, 2, 1,
            1, 2, 3,
        ],
    };
    blocks::register_block(DirtBlock);
    blocks::register_block(RotatedBlock);
    blocks::register_block(DebugBlock);
    let mut world = VoxelWorld::open(
        "ignore/test_world",
        12,
        (0, 0, 0),
        cube_sides_texarray.clone(),
        &mut commands,
        &mut meshes,
        &mut materials
    );
    let dirt = blockstate!(dirt).register();
    // world.set_block((1, 1, 1), dirt);
    // world.set_block((1, 0, 1), dirt);
    // world.set_block((1, 1, 0), dirt);
    // world.set_block((0, 0, -10), dirt);
    // for y in -8..8 {
    //     for z in -8..8 {
    //         for x in -8..8 {
    //             world.set_block((x, y, z), dirt);
    //         }
    //     }
    // }
    let rot = Quat::from_axis_angle(Vec3::Y, 0.0) * Quat::from_axis_angle(Vec3::NEG_X, 0.0);
    commands.spawn((
        Camera3dBundle {
            projection: PerspectiveProjection {
                fov: 45.0,
                aspect_ratio: 1.0,
                far: 1000.0,
                near: 0.01,
            }.into(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(rot),
            ..default()
        },
        CameraMarker
    ));

    commands.spawn(
        SpriteBundle {
            texture: asset_server.load("debug/textures/cube_sides/pos_y.png"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::Center,
                ..Default::default()
            },
            ..Default::default()
        }
    );

    let mut primary = window.get_single_mut().unwrap();
    primary.cursor.grab_mode = CursorGrabMode::Locked;
    primary.cursor.visible = false;
    commands.insert_resource(CameraRotation::default());
    commands.insert_resource(VoxelWorldRes { world });
    commands.insert_resource(CameraLocation { position: Vec3::ZERO });
}

fn update_input(
    mut evr_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera: Query<&mut Transform, With<CameraMarker>>,
    mut camrot: ResMut<CameraRotation>,
    time: Res<Time>,
    mut campos: ResMut<CameraLocation>,
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut world: ResMut<VoxelWorldRes>,
    mut gizmos: Gizmos,
) {
    // let (x, y, z) = (
    //     campos.position.x.floor() as i32,
    //     campos.position.y.floor() as i32,
    //     campos.position.z.floor() as i32,
    // );
    // world.world.move_center((x, y, z));
    if keys.just_pressed(KeyCode::Escape) {
        world.world.save_world();
        app_exit_events.send(bevy::app::AppExit);
    }
    let dt = time.delta_seconds();
    let mut mouse_motion: Vec2 = evr_motion.read()
        .map(|ev| ev.delta).sum();

    camrot.x -= mouse_motion.x * dt * 0.05;
    camrot.y += mouse_motion.y * dt * 0.05;
    let mut transform = camera.get_single_mut().unwrap();
    transform.rotation = Quat::from_axis_angle(Vec3::Y, camrot.x) * Quat::from_axis_angle(Vec3::NEG_X, camrot.y);
    
    let mut translation = Vec3::ZERO;
    const MOVE_SPEED: f32 = 7.0;
    let move_speed = if keys.pressed(KeyCode::ShiftLeft) {
        MOVE_SPEED * 5.0
    } else {
        MOVE_SPEED
    };
    if keys.pressed(KeyCode::KeyW) {
        translation += transform.forward() * dt * move_speed;
    }

    if keys.pressed(KeyCode::KeyS) {
        translation += transform.back() * dt * move_speed;
    }

    if keys.pressed(KeyCode::KeyA) {
        translation += transform.left() * dt * move_speed;
    }

    if keys.pressed(KeyCode::KeyE) {
        translation += Vec3::Y * dt * move_speed;
    }

    if keys.pressed(KeyCode::KeyQ) {
        translation += Vec3::NEG_Y * dt * move_speed;
    }

    if keys.pressed(KeyCode::KeyD) {
        translation += transform.right() * dt * move_speed;
    }
    if keys.just_pressed(KeyCode::KeyI) {
        let dirt = blockstate!(dirt).register();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let (nx, ny, nz) = (x - 8, y - 8, z - 30);
                    world.world.set_block((nx, ny, nz), dirt);
                }
            }
        }
        println!("Set Blocks");
    }
    // U for Ungage
    if keys.just_pressed(KeyCode::KeyU) {
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let (nx, ny, nz) = (x - 8, y - 8, z - 30);
                    world.world.set_block((nx, ny, nz), Id::AIR);
                }
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyT) {
        let dirt = blockstate!(dirt).register();
        world.world.render_bounds().iter().for_each(|coord| {
            world.world.set_block(coord, dirt);
        });
    }
    // if keys.just_pressed(KeyCode::KeyR) {
    //     let ray = Ray3d::new(Vec3::ZERO, Vec3::NEG_Z);
    //     if let Some((coord, id)) = world.world.raycast(ray, 100.0) {
    //         println!("Hit {id} at {coord}");
    //     }
    // }
    if let Some(RaycastResult { hit_point, coord, direction, id }) = world.world.raycast(Ray3d::new(transform.translation, transform.forward().into()), 500.0) {
        gizmos.arrow(hit_point, hit_point + Vec3::X * 0.25, Color::RED);
        gizmos.arrow(hit_point, hit_point + Vec3::Y * 0.25, Color::GREEN);
        gizmos.arrow(hit_point, hit_point + Vec3::Z * 0.25, Color::BLUE);
        if mouse_buttons.just_pressed(MouseButton::Left) {
            if let Some(direction) = direction {
                let next = coord + direction;
                world.world.set_block(next, blockstate!(dirt).register());
            } else {
                println!("No direction");
            }
        }
        if mouse_buttons.just_pressed(MouseButton::Right) {
            world.world.set_block(coord, Id::AIR);
        }
        if keys.just_pressed(KeyCode::Backspace) {
            let occlusion = world.world.get_occlusion(coord);
            println!("Occlusion: {occlusion}");
        }
        if keys.just_pressed(KeyCode::Delete) {
            println!("Hit Coord: {coord} Direction: {direction:?} Id: {id}");
        }
        if keys.just_pressed(KeyCode::End) {
            // world.world.set_block(coord, Id::AIR);
            let sect = world.world.get_section_mut(coord.section_coord()).unwrap();
            sect.blocks_dirty.mark();
            sect.section_dirty.mark();
            world.world.mark_section_dirty(coord.section_coord());
        }
    }
    transform.translation += translation;
    campos.position = transform.translation;
    
}

#[derive(Resource)]
struct CameraLocation {
    position: Vec3
}

fn update(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut render_chunks: Query<&mut Transform, With<RenderChunkMarker>>,
    mut world: ResMut<VoxelWorldRes>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // I for Ingage (lol, yes I know it's spelled wrong)
    
    let now = Instant::now();
    // let state = world.world.get_block((0,0,0));
    // world.world.set_block((0, 0, 0), if state.is_air() { blockstate!(dirt).register() } else { Id::AIR });
    world.world.talk_to_bevy(meshes, materials, render_chunks);
    let elapsed = now.elapsed();
    // println!("Frame time: {}", elapsed.as_secs_f64());
}

#[derive(Default, Resource)]
struct CameraRotation {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct CameraMarker;

#[test]
fn bitmask_test() {
    let mask = i8::bitmask_range(1..7);
    println!("{mask:08b}");
}

#[test]
fn write_read_test() -> Result<()> {
    let path: PathBuf = "ignore/test.rg".into();
    use rand::prelude::*;
    use rand::rngs::OsRng;
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);
    let mut rng = StdRng::from_seed(seed);
    {
        let mut region = RegionFile::create(&path)?;
        for z in 0..32 {
            for x in 0..32 {
                let array = Tag::from(Array::U8((0u32..4096*511+1234).map(|i| rng.gen()).collect()));
                let position = Tag::IVec2(IVec2::new(x as i32, z as i32));
                let tag = Tag::from(HashMap::from([
                    ("array".to_owned(), array.clone()),
                    ("position".to_owned(), position.clone()),
                    ("flips".to_owned(), Tag::from([Flip::X, Flip::Y, Flip::Z, Flip::X | Flip::Z, Flip::X | Flip::Z, Flip::X | Flip::Z]))
                ]));
                region.write_value((x, z), &tag)?;
            }
        }
    }
    let mut rng = StdRng::from_seed(seed);
    {
        let mut region = RegionFile::open(&path)?;
        for z in 0..32 {
            for x in 0..32 {
                let array = Box::new(Array::U8((0u32..4096*511+1234).map(|i| rng.gen()).collect()));
                let position = IVec2::new(x as i32, z as i32);
                let flips =  Tag::from([Flip::X, Flip::Y, Flip::Z, Flip::X | Flip::Z, Flip::X | Flip::Z, Flip::X | Flip::Z]);
                let read_tag: Tag = region.read_value((x, z))?;
                assert_eq!(&flips, &read_tag["flips"]);
                if let (
                    Tag::Array(read_array),
                    Tag::IVec2(read_position)
                ) = (&read_tag["array"], &read_tag["position"]) {
                    assert_eq!(&array, read_array);
                    assert_eq!(&position, read_position);
                } else {
                    panic!("Tag not read.")
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod testing_sandbox {
    use std::sync::OnceLock;

    #[derive(Debug)]
    struct OnDrop(u32);

    impl OnDrop {
        fn reset(&mut self) {
            self.0 = 0;
        }
    }

    impl Drop for OnDrop {
        fn drop(&mut self) {
            println!("Dropping id: {}", self.0);
        }
    }

    use super::*;
    #[test]
    fn sandbox() {
        static mut DATA: OnceLock<Vec<OnDrop>> = OnceLock::new();
        unsafe {
            DATA.set(Vec::new());
            let Some(data) = DATA.get_mut() else {
                panic!();
            };
            data.push(OnDrop(0));
            data.push(OnDrop(1));
            data.push(OnDrop(2));
            println!("Before First Removal");
            data[1] = OnDrop(5);
            data[2].reset();
            data[2] = OnDrop(0);
            println!("After First Removal");
            DATA.take();
            println!("After Take");
        }

    }
}

struct DirtBlock;
impl Block for DirtBlock {

    fn name(&self) -> &str {
        "dirt"
    }

    fn on_place(
            &self,
            world: &mut VoxelWorld,
            context: &mut PlaceContext,
        ) {
            // world.set_block(coord, Id::AIR);
            // println!("dirt placed: {}", context.replacement());
    }

    fn default_state(&self) -> unvoga::core::voxel::blockstate::BlockState {
        blockstate!(dirt)
    }

    fn occludee(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Occluder::FULL_FACES
    }

    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Occluder::FULL_FACES
    }

    fn push_mesh(&self, mesh_builder: &mut unvoga::core::voxel::rendering::meshbuilder::MeshBuilder, world: &VoxelWorld, coord: Coord, state: Id, occlusion: Occlusion, orientation: Orientation) {
        static MESH_DATA: LazyLock<Faces<MeshData>> = LazyLock::new(|| {
            let pos_y_mesh = MeshData {
                vertices: vec![
                    vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, -0.5),
                    vec3(-0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5),
                ],
                normals: vec![
                    Vec3::Y, Vec3::Y,
                    Vec3::Y, Vec3::Y,
                ],
                uvs: vec![
                    vec2(0.0, 0.0), vec2(1.0, 0.0),
                    vec2(0.0, 1.0), vec2(1.0, 1.0),
                ],
                texindices: vec![
                    0, 0,
                    0, 0,
                ],
                indices: vec![
                    0, 2, 1,
                    1, 2, 3,
                ],
            };
            let pos_x_mesh = pos_y_mesh.clone()
                .map_orientation(Orientation::new(Rotation::new(Direction::PosX, 0), Flip::NONE))
                .map_texindices(1);
            let pos_z_mesh = pos_y_mesh.clone()
                .map_orientation(Orientation::new(Rotation::new(Direction::PosZ, 0), Flip::NONE))
                .map_texindices(2);
            let neg_y_mesh = pos_y_mesh.clone()
                .map_orientation(Orientation::new(Rotation::new(Direction::NegY, 0), Flip::NONE))
                .map_texindices(3);
            let neg_x_mesh = pos_y_mesh.clone()
                .map_orientation(Orientation::new(Rotation::new(Direction::NegX, 0), Flip::NONE))
                .map_texindices(4);
            let neg_z_mesh = pos_y_mesh.clone()
                .map_orientation(Orientation::new(Rotation::new(Direction::NegZ, 0), Flip::NONE))
                .map_texindices(5);
            Faces::new(
                neg_x_mesh,
                neg_y_mesh,
                neg_z_mesh,
                pos_x_mesh,
                pos_y_mesh,
                pos_z_mesh
            )
        });
        if occlusion.neg_x() {
            let src_neg_x = orientation.source_face(Direction::NegX);
            mesh_builder.push_mesh_data(&MESH_DATA.face(src_neg_x));
        }
        Direction::iter().for_each(|dir| {
            if occlusion.visible(dir) {
                // get the source face because the mesh_builder will orient that face
                // to dir
                let src_face = orientation.source_face(dir);
                mesh_builder.push_mesh_data(MESH_DATA.face(dir));
            }
        });
    }
}

struct RotatedBlock;

impl RotatedBlock {
    const OCCLUDER: Occluder = Occluder {
        neg_x: OcclusionShape::Full,
        neg_y: OcclusionShape::Full,
        neg_z: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
            [0, 1],
            [1, 1],
        ])),
        pos_x: OcclusionShape::Full,
        pos_y: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
            [1, 0],
            [1, 1],
        ])),
        pos_z: OcclusionShape::Full,
    };
}

impl Block for RotatedBlock {

    fn name(&self) -> &str {
        "rotated"
    }

    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Self::OCCLUDER
    }

    fn occludee(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Self::OCCLUDER
    }

    fn default_state(&self) -> unvoga::core::voxel::blockstate::BlockState {
        blockstate!(rotated, rotation=Rotation::new(Direction::PosY, 0))
    }

    fn orientation(&self, world: &VoxelWorld, coord: Coord, state: Id) -> Orientation {
        if let StateValue::Orientation(orientation) = state["orientation"] {
            orientation
        } else {
            let rotation = if let StateValue::Rotation(rotation) = state["rotation"] {
                rotation
            } else {
                let up = if let StateValue::Direction(up) = state["up"] {
                    up
                } else {
                    Direction::PosY
                };
                let angle = if let StateValue::Int(angle) = state["angle"] {
                    angle
                } else {
                    0
                };
                Rotation::new(up, angle as i32)
            };
            let flip = if let StateValue::Flip(flip) = state["flip"] {
                flip
            } else {
                Flip::NONE
            };
            Orientation::new(rotation, flip)
        }
    }
    
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: Id, neighbor_state: Id) {
        println!("Neighbor Updated(coord = {coord:?}, neighbor_coord = {neighbor_coord:?}, neighbor_state = {neighbor_state})");
    }
}
struct DebugBlock;
impl Block for DebugBlock {
    fn name(&self) -> &str {
        "debug"
    }
    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        const OCCLUDER: Occluder = Occluder {
            neg_x: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [1, 0],
                [1, 1],
            ])),
            neg_y: OcclusionShape::Full,
            neg_z: OcclusionShape::Full,
            pos_x: OcclusionShape::Full,
            pos_y: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [1, 0],
                [1, 1],
            ])),
            pos_z: OcclusionShape::Full,
        };
        &OCCLUDER
    }
    fn default_state(&self) -> unvoga::core::voxel::blockstate::BlockState {
        blockstate!(debug)
    }
    fn call(&self, world: &mut VoxelWorld, coord: Coord, state: Id, function: &str, arg: Tag) -> Tag {
        println!("Message received: {arg:?}");
        match function {
            "test" => println!("test({arg:?})"),
            "disable" => {
                println!("Disabling.");
                world.disable(coord);
            }
            "set_enabled" => match arg {
                Tag::Bool(true) => world.enable(coord),
                Tag::Bool(false) => world.disable(coord),
                _ => println!("Invalid argument."),
            }
            other => println!("{other}({arg:?})"),
        }
        Tag::from("Debug Message Result")
    }
    fn on_place(&self, world: &mut VoxelWorld, context: &mut PlaceContext) {
        let (coord, old, new) = (
            context.coord(),
            context.old(),
            context.replacement()
        );
        println!("On Place {coord} old = {old} new = {new}");
        if matches!(context.replacement()["replace"], StateValue::Bool(true)) {
            context.replace(blockstate!(debug).register());
        } else if matches!(context.replacement()["withdata"], StateValue::Bool(true)) {
            println!("Adding data...");
            context.set_data(Tag::from("The quick brown fox jumps over the lazy dog."));
        }
    }
    fn on_remove(&self, world: &mut VoxelWorld, coord: Coord, old: Id, new: Id) {
        println!("On Remove {coord} old = {old} new = {new}");
    }
    fn on_data_set(&self, world: &mut VoxelWorld, coord: Coord, state: Id, data: &mut Tag) {
        println!("Data Set {coord} state = {state} data = {data:?}");
    }
    fn on_data_delete(&self, world: &mut VoxelWorld, coord: Coord, state: Id, data: Tag) {
        println!("Data Deleted {coord} state = {state} data = {data:?}");
    }
    fn light_updated(&self, world: &mut VoxelWorld, coord: Coord, old_level: u8, new_level: u8) {
        println!("Light Updated {coord} old = {old_level} new = {new_level}");
    }
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: Id, neighbor_state: Id) {
        println!("Neighbor Updated {coord} -> {neighbor_coord} {state} -> {neighbor_state}");
    }
    fn on_update(&self, world: &mut VoxelWorld, coord: Coord, state: Id) {
        println!("Update {coord} {state}");
        // world.set_block(coord + Direction::PosY, state);
    }
    fn enable_on_place(&self, world: &VoxelWorld, coord: Coord, state: Id) -> bool {
        matches!(state["enabled"], StateValue::Bool(true))
    }
}