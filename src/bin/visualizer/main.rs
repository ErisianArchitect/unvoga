#![allow(unused)]
use bevy::input::mouse::MouseMotion;
use bevy::render::mesh::PrimitiveTopology;
use unvoga::core::voxel::direction::Direction;
use unvoga::core::voxel::blockstate::StateValue;
// Unnamed Voxel Game

use unvoga::core::voxel::rendering::meshbuilder::MeshBuilder;
use unvoga::core::voxel::rendering::voxelmesh::MeshData;
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

const CAMERA_DISTANCE: f32 = 3.0;

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
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .add_systems(Update, menu.before(update))
        // .add_systems(Update, loading_screen.run_if(in_state(GameState::LoadingScreen)))
        // .add_systems(Update, main_menu.run_if(in_state(GameState::MainMenu)))
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
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
    let material = materials.add(VoxelMaterial {
        array_texture: cube_sides_texarray.clone(),
        light_level: 1.0,
        lightmap: vec![],
        lightmap_pad_pos_x: vec![],
        lightmap_pad_neg_x: vec![],
        lightmap_pad_pos_y: vec![],
        lightmap_pad_neg_y: vec![],
        lightmap_pad_pos_z: vec![],
        lightmap_pad_neg_z: vec![],
    });
    // Now to build the mesh. I'm going to assume the orientation code works because why not?
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
    // let mut pos_x_mesh = pos_y_mesh.clone();
    // pos_x_mesh.indices.iter_mut().for_each(|i| *i = 1);
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
    
    // mesh_builder.push_oriented_mesh_data(&top_face, Orientation::new(Rotation::new(Direction::PosY, 0), Flip::NONE));
    let builder = MeshBuilder::build(|builder| {
        builder.push_mesh_data(&pos_y_mesh);
        builder.push_mesh_data(&pos_x_mesh);
        builder.push_mesh_data(&pos_z_mesh);
        builder.push_mesh_data(&neg_y_mesh);
        builder.push_mesh_data(&neg_x_mesh);
        builder.push_mesh_data(&neg_z_mesh);
    });
    let cube_mesh: MeshData = builder.clone().into();
    commands.insert_resource(CubeMeshData { cube_mesh });
    let mut cube_mesh: Mesh = builder.into();
    let mesh_holder = MeshHolder {
        mesh: meshes.add(cube_mesh),
    };
    commands.spawn((
        MaterialMeshBundle {
            mesh: mesh_holder.mesh.clone(),
            material: material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        VisualCube,
    ));
    let rot = Quat::from_axis_angle(Vec3::Y, 45.0f32.to_radians()) * Quat::from_axis_angle(Vec3::NEG_X, 45.0f32.to_radians());
    commands.insert_resource(mesh_holder);
    commands.spawn((
        TransformBundle::from_transform(
            Transform::from_xyz(0.0, 0.0, 0.0)
                // .with_rotation(rot)
        ),
        CameraAnchor,
    )).with_children(|parent| {
        parent.spawn((
            Camera3dBundle {
                projection: PerspectiveProjection {
                    fov: 45.0,
                    aspect_ratio: 1.0,
                    far: 1000.0,
                    near: 0.01,
                }.into(),
                transform: Transform::from_xyz(0.0, 0.0, CAMERA_DISTANCE)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
        ));
    });
    commands.insert_resource(CameraRotation { x: 0.0, y: 0.0 });
    commands.insert_resource(OrientationRes::new(Orientation::default()));
    commands.insert_resource(Orientations(vec![Orientation::default()]));
    commands.insert_resource(MenuInfo::default());
}

#[derive(Resource)]
struct MenuInfo {
    in_menu: bool,
    draw_gizmos: bool,
    remap_xyz: (i32, i32, i32),
    remap_xyz_txt: (String, String, String),
    remap_xy: (i32, i32),
    remap_xy_txt: (String, String),
    remap_face: Direction,
}

impl Default for MenuInfo {
    fn default() -> Self {
        Self {
            in_menu: false,
            draw_gizmos: true,
            remap_xyz: (1, 2, 3),
            remap_xyz_txt: (String::from("1"), String::from("2"), String::from("3")),
            remap_xy: (-1, -2),
            remap_xy_txt: (String::from("-1"), String::from("-2")),
            remap_face: Direction::PosY,
        }
    }
}

#[derive(Resource)]
struct Orientations(Vec<Orientation>);

fn menu(
    mut orientation: ResMut<OrientationRes>,
    mut contexts: EguiContexts,
    mut menu_info: ResMut<MenuInfo>,
    mut orientations: ResMut<Orientations>,
) {
    use bevy_egui::egui::{self, *};
    let resp = egui::panel::SidePanel::left("left_panel").show(contexts.ctx_mut(), |ui| {
        ui.toggle_value(&mut menu_info.draw_gizmos, "Draw Gizmos");
        ui.label("Flip:");
        let mut edit = orientation.current;
        // let edit = orientation.edit();
        let (mut x, mut y, mut z) = (edit.flip.x(), edit.flip.y(), edit.flip.z());
        ui.toggle_value(&mut x, "X");
        ui.toggle_value(&mut y, "Y");
        ui.toggle_value(&mut z, "Z");
        edit.flip.set_x(x);
        edit.flip.set_y(y);
        edit.flip.set_z(z);
        let mut up = edit.rotation.up();
        ComboBox::new("top_selection", "Rotation Top")
            .selected_text(format!("{}", up))
            .show_ui(ui, |ui| {
                use unvoga::prelude::Direction;
                Direction::iter().for_each(|dir| {
                    ui.selectable_value(&mut up, dir, format!("{dir}"));
                });
            });
        let mut angle = edit.rotation.angle();
        ui.add(Slider::new(&mut angle, 0..=3));
        edit.rotation = Rotation::new(up, angle);
        if edit != orientation.current {
            orientation.set(edit);
        }
        ui.label("Remap");
        ui.horizontal(|ui| {
            ui.label("(");
            let (rect, _) = ui.allocate_exact_size(Vec2 { x: 30.0, y: ui.spacing().interact_size.y }, Sense::hover());
            let txt = TextEdit::singleline(&mut menu_info.remap_xyz_txt.0);
            ui.put(rect, txt);
            ui.label(",");
            let (rect, _) = ui.allocate_exact_size(Vec2 { x: 30.0, y: ui.spacing().interact_size.y }, Sense::hover());
            let txt = TextEdit::singleline(&mut menu_info.remap_xyz_txt.1);
            ui.put(rect, txt);
            ui.label(",");
            let (rect, _) = ui.allocate_exact_size(Vec2 { x: 30.0, y: ui.spacing().interact_size.y }, Sense::hover());
            let txt = TextEdit::singleline(&mut menu_info.remap_xyz_txt.2);
            ui.put(rect, txt);
            ui.label(")");
        });
        let x_n = i32::from_str_radix(&menu_info.remap_xyz_txt.0, 10);
        let y_n = i32::from_str_radix(&menu_info.remap_xyz_txt.1, 10);
        let z_n = i32::from_str_radix(&menu_info.remap_xyz_txt.2, 10);
        match (x_n, y_n, z_n) {
            (Ok(x), Ok(y), Ok(z)) => {
                let (x, y, z) = edit.transform((x, y, z));
                ui.label(format!("({x}, {y}, {z})"));
            }
            _ => {
                ui.label("Error");
            },
        }
        ComboBox::new("remap_face", "Remap Face")
            .selected_text(format!("{}", menu_info.remap_face))
            .show_ui(ui, |ui| {
                use unvoga::prelude::Direction;
                Direction::iter().for_each(|dir| {
                    ui.selectable_value(&mut menu_info.remap_face, dir, format!("{dir}"));
                });
            });
        let src_face = edit.source_face(menu_info.remap_face);
        ui.label(format!("Source Face: {src_face}"));
        ui.horizontal(|ui| {
            ui.label("(");
            let (rect, _) = ui.allocate_exact_size(Vec2 { x: 30.0, y: ui.spacing().interact_size.y }, Sense::hover());
            let txt = TextEdit::singleline(&mut menu_info.remap_xy_txt.0);
            ui.put(rect, txt);
            ui.label(",");
            let (rect, _) = ui.allocate_exact_size(Vec2 { x: 30.0, y: ui.spacing().interact_size.y }, Sense::hover());
            let txt = TextEdit::singleline(&mut menu_info.remap_xy_txt.1);
            ui.put(rect, txt);
            ui.label(")");
        });
        let x_n = i32::from_str_radix(&menu_info.remap_xy_txt.0, 10);
        let y_n = i32::from_str_radix(&menu_info.remap_xy_txt.1, 10);
        match (x_n, y_n) {
            (Ok(x), Ok(y)) => {
                let (x, y) = edit.source_face_coord(menu_info.remap_face, (x, y));
                ui.label(format!("({x}, {y})"));
            }
            _ => {
                ui.label("Error");
            },
        }

        if ui.button("⊞").clicked() {
            orientations.0.push(edit);
        }
        let mut remove = None;
        orientations.0.iter().enumerate().for_each(|(index, orient)| {
            ui.group(|ui| {
                fn select_color(predicate: bool) -> Color32 {
                    if predicate {
                        Color32::GREEN
                    } else {
                        Color32::RED
                    }
                }
                let mut layout = egui::text::LayoutJob::default();
                RichText::new("X").color(select_color(orient.flip.x())).monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                RichText::new("Y").color(select_color(orient.flip.y())).monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                RichText::new("Z").color(select_color(orient.flip.z())).monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                RichText::new(" Up: ").monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                RichText::new(format!("{}", orient.rotation.up())).color(Color32::WHITE).monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                RichText::new(" Angle: ").monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                RichText::new(format!("{}", orient.rotation.angle())).color(Color32::WHITE).monospace().append_to(&mut layout, ui.style(), FontSelection::FontId(FontId::monospace(16.0)), Align::Center);
                ui.horizontal(|ui| {
                    if ui.button(layout).clicked() {
                        orientation.set(*orient);
                    }
                    if ui.button("❌").clicked() {
                        remove.replace(index);
                    }
                })
            });
        });
        if let Some(index) = remove {
            orientations.0.remove(index);
        }
    }).response;
    menu_info.in_menu = resp.has_focus() || resp.contains_pointer();

}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rotation: ResMut<CameraRotation>,
    time: Res<Time>,
    mesh_holder: Res<MeshHolder>,
    cube_mesh: Res<CubeMeshData>,
    mut anchor: Query<&mut Transform,With<CameraAnchor>>,
    mut evr_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut orientation: ResMut<OrientationRes>,
    mut gizmos: Gizmos,
    menu_info: Res<MenuInfo>,
) {
    
    if !menu_info.in_menu {
        let new_orient = orientation.edit();
        if keys.just_pressed(KeyCode::KeyE) {
            let rot = new_orient.rotation;
            new_orient.rotation = rot.cycle(1);
        }
        if keys.just_pressed(KeyCode::KeyQ) {
            let rot = new_orient.rotation;
            new_orient.rotation = rot.cycle(-1);
        }
        if keys.just_pressed(KeyCode::KeyX) {
            let rot = new_orient.rotation;
            new_orient.flip.invert_x();
        }
        if keys.just_pressed(KeyCode::KeyY) {
            let rot = new_orient.rotation;
            new_orient.flip.invert_y();
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            let rot = new_orient.rotation;
            new_orient.flip.invert_z();
        }
        if mouse_buttons.pressed(MouseButton::Left) {
            let delta = time.delta_seconds();
            let mut transform = anchor.get_single_mut().expect("Failed to get anchor.");
            let mut mouse_motion: Vec2 = evr_motion.read()
                .map(|ev| ev.delta).sum();
            rotation.x -= mouse_motion.x * delta;
            rotation.y += mouse_motion.y * delta;
            transform.rotation = Quat::from_axis_angle(Vec3::Y, rotation.x) * Quat::from_axis_angle(Vec3::NEG_X, rotation.y);
        }
    }
    if let Some(orient) = orientation.update() {
        let mesh = meshes.get_mut(mesh_holder.mesh.id()).expect("Failed to get the mesh");
        MeshBuilder::build_mesh(mesh, |build| {
            build.push_oriented_mesh_data(&cube_mesh.cube_mesh, orient);
        });
    }
    const GIZLEN: f32 = 0.2;
    if menu_info.draw_gizmos {
        gizmos.arrow(Vec3::ZERO, Vec3::ZERO + Vec3::X * GIZLEN, Color::RED);
        gizmos.arrow(Vec3::ZERO, Vec3::ZERO + Vec3::Y * GIZLEN, Color::GREEN);
        gizmos.arrow(Vec3::ZERO, Vec3::ZERO + Vec3::Z * GIZLEN, Color::BLUE);
    }
}

#[derive(Resource)]
struct OrientationRes {
    current: Orientation,
    new_orientation: Orientation,
}

impl OrientationRes {
    fn new(orientation: Orientation) -> Self {
        Self {
            current: orientation,
            new_orientation: orientation,
        }
    }

    pub fn set(&mut self, orientation: Orientation) {
        self.new_orientation = orientation;
    }

    pub fn edit(&mut self) -> &mut Orientation {
        &mut self.new_orientation
    }

    pub fn update(&mut self) -> Option<Orientation> {
        if self.current == self.new_orientation {
            return None;
        }
        self.current = self.new_orientation;
        Some(self.new_orientation)
    }
}

#[derive(Resource)]
struct CameraRotation {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct VisualCube;

#[derive(Resource)]
struct CubeMeshData {
    cube_mesh: MeshData,
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