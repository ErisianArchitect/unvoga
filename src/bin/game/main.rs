#![allow(unused)]
use unvoga::core::voxel::direction::Direction;
// Unnamed Voxel Game

use unvoga::prelude::*;
use unvoga::core::voxel::{block::Block, blocks::{self, Id}, coord::Coord, faces::Faces, occlusionshape::OcclusionShape, rendering::voxelmaterial::VoxelMaterial, tag::Tag, world::{occlusion::Occlusion, VoxelWorld}};
use std::fmt::{Debug, Display};

use bevy::{
    asset::LoadState, math::{vec2, vec3, vec4}, prelude::*, render::{camera::ScalingMode, mesh::{Indices, MeshVertexAttribute, MeshVertexAttributeId}, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, VertexFormat}, texture::ImageSampler}, window::PresentMode
};

// use bevy::ecs::component::Component;
use bevy_egui::{EguiContexts, EguiPlugin};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    LoadingScreen,
    MainMenu,
    SinglePlayer,
    // MultiPlayer,
}

mod cleanup {
    // use bevy::ecs::component::Component;
    use super::*;

    #[derive(Component)]
    pub struct Menu;
    #[derive(Component)]
    pub struct SinglePlayer;
    #[derive(Component)]
    pub struct LoadingScreen;
}

fn main() {
    // sandbox::sandbox();
    // return;
    // TODO: Read from configuration file.
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
        .insert_resource(LoadingState { tile_stack_texture: None })
        .insert_state(GameState::LoadingScreen)
        .add_systems(Update, loading_screen.run_if(in_state(GameState::LoadingScreen)))
        .add_systems(Update, main_menu.run_if(in_state(GameState::MainMenu)))
        .add_systems(OnEnter(GameState::LoadingScreen), enter_loading_screen)
        .add_systems(OnExit(GameState::LoadingScreen), cleanup_system::<cleanup::LoadingScreen>)
        .add_systems(OnEnter(GameState::MainMenu), on_enter_main_menu)
        .add_systems(OnExit(GameState::MainMenu), cleanup_system::<cleanup::Menu>)
        .add_systems(OnEnter(GameState::SinglePlayer), on_enter_singleplayer)
        .add_systems(OnExit(GameState::SinglePlayer), cleanup_system::<cleanup::SinglePlayer>)
        .insert_resource(Assets::<VoxelMaterial>::default())
        .insert_resource(Assets::<Mesh>::default())
        .insert_resource(ClearColor(Color::rgb(0.2,0.2,0.2)))
        .insert_resource(Msaa::Off)
        .run();
}

#[derive(Resource)]
struct LoadingState {
    tile_stack_texture: Option<LoadingImage>,
}

impl LoadingState {
    pub fn is_loaded(&self) -> bool {
        if let Some(tex) = &self.tile_stack_texture {
            if !tex.loaded() {
                return false;
            }
        }
        true
        // As more stuff is added to LoadingState, add the checks for it here.
    }
}

pub struct LoadingImage {
    handle: Handle<Image>,
    loaded: bool,
}

impl LoadingImage {
    pub fn new(handle: Handle<Image>) -> Self {
        Self {
            handle,
            loaded: false,
        }
    }

    pub fn mark_loaded(&mut self) {
        self.loaded = true;
    }

    pub fn loaded(&self) -> bool {
        self.loaded
    }

    pub fn handle(&self) -> &Handle<Image> {
        &self.handle
    }

    pub fn handle_mut(&mut self) -> &mut Handle<Image> {
        &mut self.handle
    }

    pub fn take(self) -> Handle<Image> {
        self.handle
    }
}

struct LoadingTexture {
    is_loaded: bool,
    image: Handle<Image>,
}

fn enter_loading_screen(
    mut load_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
) {
    load_state.tile_stack_texture = Some(
        // LoadingImage::new(asset_server.load("textures/atlases/mc_tile_stack.png"))
        LoadingImage::new(asset_server.load("textures/atlases/tile_stack.png"))
    );

}

#[derive(Resource)]
pub struct VoxelWorldResources {
    texture_array: Handle<Image>,
}

impl VoxelWorldResources {
    pub fn new(
        texture_array: Handle<Image>,
    ) -> Self {
        Self {
            texture_array,
        }
    }

    pub fn texture_array(&self) -> Handle<Image> {
        self.texture_array.clone()
    }
}


fn loading_screen(
    mut commands: Commands,
    mut load_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(tex) = &mut load_state.tile_stack_texture {
        if !tex.loaded()
        && asset_server.load_state(tex.handle().id()) == LoadState::Loaded {
            tex.mark_loaded();
            let handle = tex.handle().clone();
            let mut image = images.get_mut(&handle).expect("Expected the handle to be valid.");
            // This makes it so that the textures aren't blurry.
            image.sampler = ImageSampler::nearest();
            image.reinterpret_stacked_2d_as_array(16);
            commands.remove_resource::<LoadingState>();
            commands.insert_resource(VoxelWorldResources::new(handle));
            commands.insert_resource(VoxelData {vox_mat: None  });
        }
    }
    if load_state.is_loaded() {
        next_state.set(GameState::MainMenu);
    }
}

fn main_menu(
    mut commands: Commands,
    mut contexts: EguiContexts,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    voxel_resources: Res<VoxelWorldResources>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut movers: Query<&mut Transform, With<Mover>>,
    mut vox_data: ResMut<VoxelData>,
    mut cursor_event: EventReader<CursorMoved>,
) {
    // for mut mover in movers.iter_mut() {
    //     mover.translation.x += 0.001;
    // }
    for event in cursor_event.read() {
        // println!("Move to {:?}", event.position);
    }
    use bevy_egui::egui::{self, *};
    egui::Window::new("Debug")
        .show(contexts.ctx_mut(), |ui| {
            // for mover in movers.iter() {
                //     ui.label(format!("X: {}", mover.translation.x));
                // }
            if ui.button("Break").clicked() {
                next_state.set(GameState::SinglePlayer);
            }
            if let Some(vox_mat) = &vox_data.vox_mat {
                if let Some(material) = voxel_materials.get_mut(vox_mat.id()) {
                    if ui.button("Randomize (0,0)").clicked() {
                        fn mapindex(x: usize, y: usize) -> usize {
                            x | (y << 4)
                        }
                        for i in (0..256) {
                            let level = rand::random::<u8>().rem_euclid(16) as f32 / 15.0;
                            material.lightmap[i] = level;
                        }
                        // material.lightmap[mapindex(0, 0)] = rand::random::<f32>().rem_euclid(1.0);
                        // material.lightmap[mapindex(1, 0)] = rand::random::<f32>().rem_euclid(1.0);
                        // material.lightmap[mapindex(0, 1)] = rand::random::<f32>().rem_euclid(1.0);
                        // material.lightmap[mapindex(1, 1)] = rand::random::<f32>().rem_euclid(1.0);
                    }
                }
            }
            if let Ok(mut mover) = movers.get_single_mut() {
                
                ui.label(format!("X: {}", mover.translation.x));
                ui.horizontal(|ui| {
                    if ui.button("Move Left").clicked() {
                        mover.translation.x -= 0.1;
                    }
                    if ui.button("Move Right").clicked() {
                        mover.translation.x += 0.1;
                    }
                });
                if ui.button("Move Up").clicked() {
                    mover.translation.z -= 0.1;
                }
                if ui.button("Move Down").clicked() {
                    mover.translation.z += 0.1;
                }
            }
        });
    // egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
    //     if ui.button("Add Mesh").clicked() {
            
            
    //     }
    //     if entered.marked() {
    //         ui.label("Entered Main Menu");
    //     } else {
    //         ui.label("Not Entered Main Menu");
    //     }
    // });
}

#[derive(Component)]
pub struct Mover;

#[derive(Resource)]
pub struct VoxelData {
    vox_mat: Option<Handle<VoxelMaterial>>,
}

#[derive(Resource)]
pub struct MeshHolder {
    mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct CameraAnchor;

fn on_enter_main_menu(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    voxel_resources: Res<VoxelWorldResources>,
) {
    let side_texture_paths = vec![
        "./assets/debug/textures/cube_sides/pos_y.png",
        "./assets/debug/textures/cube_sides/pos_x.png",
        "./assets/debug/textures/cube_sides/pos_z.png",
        "./assets/debug/textures/cube_sides/neg_y.png",
        "./assets/debug/textures/cube_sides/neg_x.png",
        "./assets/debug/textures/cube_sides/neg_z.png",
    ];
    let cube_sides_texarray = images.add(unvoga::core::util::texture_array::create_texture_array_from_paths(512, 512, side_texture_paths).expect("Failed to create texture array."));
    let mut mesh = bevy::prelude::Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    let vertices: Vec<_> = [
        vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5),
        
    ].into_iter()
    // .map(|v| v + vec3(-0.5, 0.0, -0.5))
    .collect();
    let normals: Vec<_> = vec![
        Vec3::Y,Vec3::Y,
        Vec3::Y,Vec3::Y,
    ];
    // let uvs = vec![
    //     vec3(0.0, 0.0, 0.0), vec3(0.0625, 0.0, 0.0),
    //     vec3(0.0, 0.0, 1.0), vec3(0.0625, 0.0, 0.0625),
    // ];
    let uvs = vec![
        vec2(0.0, 0.0), vec2(1.0, 0.0),
        vec2(0.0, 1.0), vec2(1.0, 1.0),
    ];
    let tex_indices: Vec<u32> = (0..4).map(|i| if i < 4 { 0 } else { 0 }).collect();
    let indices = Indices::U32(vec![
        0, 2, 1, 1, 2, 3,// First
        // 4, 6, 5, 5, 6, 7,// Second
    ]);

    mesh.insert_attribute(MeshVertexAttribute::new("position", 0, VertexFormat::Float32x3), vertices);
    mesh.insert_attribute(MeshVertexAttribute::new("uv", 1, VertexFormat::Float32x2), uvs);
    mesh.insert_attribute(MeshVertexAttribute::new("normal", 2, VertexFormat::Float32x3), normals);
    mesh.insert_attribute(MeshVertexAttribute::new("texindex", 3, VertexFormat::Uint32), tex_indices);
    mesh.insert_indices(indices);
    let mut trans = Transform::from_xyz(0.0, 0.0, 0.0);
    // trans.rotate_axis(Vec3::Y, 45.0);
    let mut lightmap: Vec<f32> = (0..256).map(|i| {
        let x = i & 0xf;
        let y = (i & 0xf0) >> 4;
        if x == 15
        // || y == 0
        // || x == 15
        || y == 15
        {
            return 0.0;
        }
        // if i == 0 {
        //     return 1.0;
        // }
        // let x = i & 0xf;
        // let xf = x as f32;
        // let shade = (xf.rem_euclid(16.0)) / 16.0;
        // shade
        let shade: f32 = rand::random();
        shade.rem_euclid(1.0)
    }).collect();
    let vox_mat = materials.add(VoxelMaterial { 
        array_texture: cube_sides_texarray,
        light_level: 1.0,
        lightmap,
        lightmap_pad_pos_x: vec![],
        lightmap_pad_neg_x: vec![],
        lightmap_pad_pos_y: vec![],
        lightmap_pad_neg_y: vec![],
        lightmap_pad_pos_z: vec![],
        lightmap_pad_neg_z: vec![],
    });
    // vox_data.as_mut().vox_mat = Some(vox_mat.clone());
    let mesh_holder = MeshHolder {
        mesh: meshes.add(mesh),
    };
    commands.spawn((
        MaterialMeshBundle {
            mesh: mesh_holder.mesh.clone(),
            material: vox_mat,
            transform: trans,
            ..default()
        },
        cleanup::Menu,
        Mover,
    ));
    commands.insert_resource(mesh_holder);
    // let mat = std_materials.add(StandardMaterial {
    //     base_color_texture: Some(asset_server.load("textures/atlases/mc_tilestack.png").into()),
    //     ..Default::default()
    // });
    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(mesh),
    //         material: mat,
    //         ..default()
    //     },
    //     cleanup::Menu
    // ));
    // Camera
    let cam_rot = Quat::from_axis_angle(Vec3::Y, 45.0f32.to_radians()) * Quat::from_axis_angle(Vec3::NEG_X, 25.0f32.to_radians());
    //Quat::from_euler(EulerRot::XYZ, -45.0f32.to_radians(), 45.0f32.to_radians(), 0.0)
    commands.spawn((
        TransformBundle::from_transform(
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(cam_rot)
        ),
        CameraAnchor
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3dBundle {
                    // projection: OrthographicProjection {
                    //     scaling_mode: ScalingMode::FixedVertical(32.0),
                    //     ..Default::default()
                    // }.into(),
                    projection: PerspectiveProjection {
                        fov: 45.0,
                        aspect_ratio: 1.0,
                        far: 1000.0,
                        near: 0.01,
                    }.into(),
                    transform: Transform::from_xyz(0.0, 0.0, 5.0)
                        .looking_at(vec3(0.0, 0.0, 0.0), Vec3::Y),
                    ..default()
                },
                cleanup::Menu
            ));
        });
    // commands.spawn((
    //     Camera3dBundle {
    //         // projection: OrthographicProjection {
    //         //     scaling_mode: ScalingMode::FixedVertical(32.0),
    //         //     ..Default::default()
    //         // }.into(),
    //         projection: PerspectiveProjection {
    //             fov: 45.0,
    //             aspect_ratio: 1.0,
    //             far: 1000.0,
    //             near: 0.01,
    //         }.into(),
    //         transform: Transform::from_xyz(0.0, 5.0, 0.0)
    //             .looking_at(vec3(0.0, 0.0, 0.0), -Vec3::Z)
    //             ,
    //         ..default()
    //     },
    //     cleanup::Menu
    // ));
    // commands.spawn((
    //     Camera3dBundle {
    //         projection: PerspectiveProjection {
    //             fov: 60.0f32.to_radians(),
    //             near: 0.01,
    //             far: 100000.0,
    //             aspect_ratio: 1280.0 / 720.0,
    //         }.into(),
    //         transform: Transform::from_xyz(0.0, 10.0, 0.001)
    //             .looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     },
    //     cleanup::Menu
    // ));
}

fn cleanup_system<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn on_enter_singleplayer(mut commands: Commands) {

}

fn on_exit_singleplayer(mut commands: Commands) {

}