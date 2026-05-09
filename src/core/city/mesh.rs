use bevy::{
    math::{vec2, vec3, Vec2, Vec3},
    prelude::Mesh,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

use super::tile::{Building, Road};

pub fn road_mesh(road: &Road) -> Option<Mesh> {
    if road.centerline.len() < 2 {
        return None;
    }

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut distance = 0.0f32;
    let half_width = road.width_m * 0.5;

    for (segment_index, segment) in road.centerline.windows(2).enumerate() {
        let a = segment[0];
        let b = segment[1];
        let ab = Vec2::new((b.x - a.x) as f32, (b.z - a.z) as f32);
        let length = ab.length();
        if length <= f32::EPSILON {
            continue;
        }

        let dir = ab / length;
        let normal = Vec2::new(-dir.y, dir.x) * half_width;
        let start = vertices.len() as u32;
        vertices.extend([
            vec3(a.x as f32 + normal.x, 0.03, a.z as f32 + normal.y),
            vec3(a.x as f32 - normal.x, 0.03, a.z as f32 - normal.y),
            vec3(b.x as f32 + normal.x, 0.03, b.z as f32 + normal.y),
            vec3(b.x as f32 - normal.x, 0.03, b.z as f32 - normal.y),
        ]);
        normals.extend([Vec3::Y; 4]);
        uvs.extend([
            vec2(0.0, distance),
            vec2(1.0, distance),
            vec2(0.0, distance + length),
            vec2(1.0, distance + length),
        ]);
        indices.extend([start, start + 2, start + 1, start + 1, start + 2, start + 3]);
        distance += length;

        if segment_index > 0 {
            // Slight overlap hides tiny cracks at polyline segment joins.
            let join = road.centerline[segment_index];
            push_flat_disc(
                &mut vertices,
                &mut normals,
                &mut uvs,
                &mut indices,
                vec3(join.x as f32, 0.031, join.z as f32),
                half_width,
                8,
            );
        }
    }

    if vertices.is_empty() {
        return None;
    }
    Some(build_mesh(vertices, normals, uvs, indices))
}

pub fn building_mesh(building: &Building) -> Option<Mesh> {
    let footprint = &building.footprint;
    if footprint.len() < 3 {
        return None;
    }

    let bottom = building.min_height_m;
    let top = building.min_height_m + building.height_m.max(0.1);
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let top_start = vertices.len() as u32;
    for point in footprint {
        vertices.push(vec3(point.x as f32, top, point.z as f32));
        normals.push(Vec3::Y);
        uvs.push(vec2(point.x as f32, point.z as f32));
    }
    for i in 1..footprint.len() - 1 {
        indices.extend([top_start, top_start + i as u32, top_start + i as u32 + 1]);
    }

    let bottom_start = vertices.len() as u32;
    for point in footprint {
        vertices.push(vec3(point.x as f32, bottom, point.z as f32));
        normals.push(Vec3::NEG_Y);
        uvs.push(vec2(point.x as f32, point.z as f32));
    }
    for i in 1..footprint.len() - 1 {
        indices.extend([
            bottom_start,
            bottom_start + i as u32 + 1,
            bottom_start + i as u32,
        ]);
    }

    for i in 0..footprint.len() {
        let j = (i + 1) % footprint.len();
        let a = footprint[i];
        let b = footprint[j];
        let edge = Vec2::new((b.x - a.x) as f32, (b.z - a.z) as f32);
        if edge.length_squared() <= f32::EPSILON {
            continue;
        }
        let side_normal = Vec3::new(edge.y, 0.0, -edge.x).normalize();
        let start = vertices.len() as u32;
        vertices.extend([
            vec3(a.x as f32, bottom, a.z as f32),
            vec3(b.x as f32, bottom, b.z as f32),
            vec3(a.x as f32, top, a.z as f32),
            vec3(b.x as f32, top, b.z as f32),
        ]);
        normals.extend([side_normal; 4]);
        let edge_len = edge.length();
        uvs.extend([
            vec2(0.0, 0.0),
            vec2(edge_len, 0.0),
            vec2(0.0, building.height_m),
            vec2(edge_len, building.height_m),
        ]);
        indices.extend([start, start + 2, start + 1, start + 1, start + 2, start + 3]);
    }

    Some(build_mesh(vertices, normals, uvs, indices))
}

fn push_flat_disc(
    vertices: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    indices: &mut Vec<u32>,
    center: Vec3,
    radius: f32,
    segments: u32,
) {
    let start = vertices.len() as u32;
    vertices.push(center);
    normals.push(Vec3::Y);
    uvs.push(vec2(0.5, 0.5));
    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let point = center + vec3(angle.cos() * radius, 0.0, angle.sin() * radius);
        vertices.push(point);
        normals.push(Vec3::Y);
        uvs.push(vec2(angle.cos() * 0.5 + 0.5, angle.sin() * 0.5 + 0.5));
    }
    for i in 0..segments {
        indices.extend([start, start + 1 + i, start + 1 + ((i + 1) % segments)]);
    }
}

fn build_mesh(vertices: Vec<Vec3>, normals: Vec<Vec3>, uvs: Vec<Vec2>, indices: Vec<u32>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::city::{
        generate_demo_tile, Building, CityTileId, DemoCityConfig, GeoPoint, Road, RoadClass,
        SourceTag,
    };

    #[test]
    fn road_mesh_builds_for_demo_road() {
        let tile = generate_demo_tile(
            CityTileId::new(0, 0, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        );

        assert!(road_mesh(&tile.roads[0]).is_some());
    }

    #[test]
    fn building_mesh_builds_for_demo_building() {
        let tile = generate_demo_tile(
            CityTileId::new(0, 0, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        );

        assert!(building_mesh(&tile.buildings[0]).is_some());
    }

    #[test]
    fn invalid_features_do_not_build_meshes() {
        assert!(road_mesh(&Road {
            id: "empty".to_owned(),
            class: RoadClass::Local,
            centerline: Vec::new(),
            width_m: 1.0,
            lanes: 1,
            source: SourceTag::Procedural,
        })
        .is_none());

        assert!(building_mesh(&Building {
            id: "empty".to_owned(),
            footprint: Vec::new(),
            min_height_m: 0.0,
            height_m: 10.0,
            levels: None,
            source: SourceTag::Procedural,
        })
        .is_none());
    }
}
