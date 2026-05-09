use std::collections::HashMap;

use bevy::{
    app::AppExit,
    input::mouse::MouseMotion,
    math::{vec2, vec3},
    prelude::*,
    window::PresentMode,
};
use unvoga::{
    core::city::{
        building_mesh, generate_demo_tile, road_mesh, Building, CityTile, CityTileId,
        DemoCityConfig, GeoPoint, LocalPoint, Road, RoadClass,
    },
    game::cameras::{CameraContoller, CameraType},
};

const TILE_RADIUS: i32 = 1;

#[derive(Component)]
struct CityCamera;

#[derive(Component)]
struct CityFeature;

#[derive(Resource)]
struct CityCameraController(CameraContoller);

#[derive(Resource)]
struct CityStreaming {
    origin: GeoPoint,
    config: DemoCityConfig,
    loaded_tiles: HashMap<CityTileId, Vec<Entity>>,
}

fn main() {
    if std::env::var_os("BEVY_ASSET_ROOT").is_none()
        && std::env::var_os("CARGO_MANIFEST_DIR").is_none()
    {
        unsafe {
            std::env::set_var("BEVY_ASSET_ROOT", env!("CARGO_MANIFEST_DIR"));
        }
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vibevoga City Viewer".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.12)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 650.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(CityCameraController(CameraContoller::new(
            CameraType::Free,
            vec2(0.78, -0.48),
            80.0,
            0.12,
        )))
        .insert_resource(CityStreaming {
            origin: GeoPoint::new(25.7617, -80.1918),
            config: DemoCityConfig::default(),
            loaded_tiles: HashMap::new(),
        })
        .add_systems(Startup, setup_city)
        .add_systems(
            Update,
            (exit_on_escape, city_camera_controls, stream_city_tiles).chain(),
        )
        .run();
}

fn setup_city(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut streaming: ResMut<CityStreaming>,
) {
    sync_city_tiles(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut streaming,
        CityTileId::new(0, 0, 0),
    );

    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.8, 0.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            far: 5_000.0,
            ..default()
        }),
        Transform::from_xyz(255.0, 260.0, 710.0).looking_at(vec3(255.0, 0.0, 255.0), Vec3::Y),
        CityCamera,
    ));

    spawn_debug_marker(&mut commands, &mut meshes, &mut materials);
    info!("city_viewer loaded {} tiles", streaming.loaded_tiles.len());
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tile: &CityTile,
) -> Entity {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.38, 0.29),
        unlit: true,
        perceptual_roughness: 0.9,
        ..default()
    });
    let center = tile_center(tile.min_local(), tile.max_local());
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(
                tile.tile_size_m as f32,
                0.08,
                tile.tile_size_m as f32,
            ))),
            MeshMaterial3d(material),
            Transform::from_xyz(center.x as f32, -0.06, center.z as f32),
            CityFeature,
        ))
        .id()
}

fn spawn_road(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    road: &Road,
) -> Vec<Entity> {
    let Some(mesh) = road_mesh(road) else {
        return Vec::new();
    };
    let material = materials.add(StandardMaterial {
        base_color: road_color(&road.class),
        unlit: true,
        perceptual_roughness: 0.96,
        ..default()
    });

    vec![commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            CityFeature,
        ))
        .id()]
}

fn spawn_building(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    building: &Building,
) -> Option<Entity> {
    let mesh = building_mesh(building)?;
    let height = building.height_m.max(1.0);
    let color_t = (height / 90.0).clamp(0.0, 1.0);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(
            0.38 + 0.18 * color_t,
            0.39 + 0.12 * color_t,
            0.42 + 0.2 * color_t,
        ),
        unlit: true,
        perceptual_roughness: 0.82,
        ..default()
    });

    Some(
        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                CityFeature,
            ))
            .id(),
    )
}

fn spawn_debug_marker(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.15, 0.05),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 120.0, 16.0))),
        MeshMaterial3d(material),
        Transform::from_xyz(255.0, 60.0, 255.0),
        CityFeature,
    ));
}

fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn stream_city_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut streaming: ResMut<CityStreaming>,
    camera: Query<&Transform, With<CityCamera>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let center = CityTileId::from_local(
        LocalPoint::new(camera.translation.x as f64, camera.translation.z as f64),
        streaming.config.tile_size_m,
        0,
    );
    sync_city_tiles(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut streaming,
        center,
    );
}

fn sync_city_tiles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    streaming: &mut CityStreaming,
    center: CityTileId,
) {
    let mut unload = Vec::new();
    for id in streaming.loaded_tiles.keys().copied() {
        if (id.x - center.x).abs() > TILE_RADIUS || (id.z - center.z).abs() > TILE_RADIUS {
            unload.push(id);
        }
    }
    for id in unload {
        if let Some(entities) = streaming.loaded_tiles.remove(&id) {
            for entity in entities {
                commands.entity(entity).despawn();
            }
        }
    }

    for z in center.z - TILE_RADIUS..=center.z + TILE_RADIUS {
        for x in center.x - TILE_RADIUS..=center.x + TILE_RADIUS {
            let id = CityTileId::new(x, z, center.lod);
            if streaming.loaded_tiles.contains_key(&id) {
                continue;
            }
            let tile = generate_demo_tile(id, streaming.origin, streaming.config);
            let entities = spawn_city_tile(commands, meshes, materials, &tile);
            streaming.loaded_tiles.insert(id, entities);
        }
    }
}

fn spawn_city_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tile: &CityTile,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    entities.push(spawn_ground(commands, meshes, materials, tile));
    for road in &tile.roads {
        entities.extend(spawn_road(commands, meshes, materials, road));
    }
    for building in &tile.buildings {
        if let Some(entity) = spawn_building(commands, meshes, materials, building) {
            entities.push(entity);
        }
    }
    entities
}

fn city_camera_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut controller: ResMut<CityCameraController>,
    mut camera: Query<&mut Transform, With<CityCamera>>,
) {
    let Ok(transform) = camera.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let mut controller = controller.0.begin_transform(transform);
    let rotation = mouse_motion
        .read()
        .fold(Vec2::ZERO, |acc, event| acc + event.delta);
    controller.rotate(rotation, dt, 1.0);

    let mut movement = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        movement.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        movement.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyQ) {
        movement.y -= 1.0;
    }
    let multiplier = if keys.pressed(KeyCode::ShiftLeft) {
        3.0
    } else {
        1.0
    };
    controller.translate(movement, dt, multiplier);
}

fn road_color(class: &RoadClass) -> Color {
    match class {
        RoadClass::Motorway => Color::srgb(0.18, 0.18, 0.19),
        RoadClass::Arterial => Color::srgb(0.20, 0.20, 0.21),
        RoadClass::Collector => Color::srgb(0.23, 0.23, 0.24),
        RoadClass::Local => Color::srgb(0.26, 0.26, 0.27),
        RoadClass::Service => Color::srgb(0.30, 0.30, 0.31),
        RoadClass::Footway => Color::srgb(0.44, 0.42, 0.38),
    }
}

fn tile_center(min: LocalPoint, max: LocalPoint) -> LocalPoint {
    LocalPoint::new((min.x + max.x) * 0.5, (min.z + max.z) * 0.5)
}
