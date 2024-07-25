#![allow(unused)]
use bevy::{math::Vec2, math::Vec3, render::{mesh::{Indices, Mesh, MeshVertexAttribute}, render_asset::RenderAssetUsages, render_resource::VertexFormat}};


pub struct MeshBuilder {
    vertices: Vec<Vec3>,
    uvs: Vec<Vec2>,
    texture_indices: Vec<u32>,
    indices: Vec<u32>,
}

pub struct MesherCapacities {
    pub vertices: usize,
    pub uvs: usize,
    pub texture_indices: usize,
    pub indices: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex {
    vertex: Vec3,
    uv: Vec2,
    texture_index: u32,
}

impl VoxelVertex {
    pub const fn new(
        vertex: Vec3,
        uv: Vec2,
        texture_index: u32,
    ) -> Self {
        Self {
            vertex,
            uv,
            texture_index,
        }
    }
}

pub trait VoxelVertexItem {
    fn vertex(self) -> VoxelVertex;
}

impl VoxelVertexItem for VoxelVertex {
    fn vertex(self) -> VoxelVertex {
        self
    }
}

impl VoxelVertexItem for &VoxelVertex {
    fn vertex(self) -> VoxelVertex {
        self.clone()
    }
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            uvs: Vec::new(),
            texture_indices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn push_quad<V: VoxelVertexItem>(&mut self, quad: impl IntoIterator<Item = V>) {
        let start_index = self.vertices.len() as u32;
        quad.into_iter()
            .take(4)
            .map(|v| v.vertex())
            .for_each(|VoxelVertex { vertex, uv, texture_index }| {
                self.vertices.push(vertex);
                self.uvs.push(uv);
                self.texture_indices.push(texture_index);
            });
        const QUAD_INDICES: [u32; 6] = [0, 2, 1, 1, 2, 3];
        self.indices.extend(QUAD_INDICES.into_iter().map(|i| start_index + i));
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        let Self { vertices, uvs, texture_indices, indices } = self;
        mesh.insert_attribute(MeshVertexAttribute::new("position", 0, VertexFormat::Float32x3), vertices);
        mesh.insert_attribute(MeshVertexAttribute::new("uv", 1, VertexFormat::Float32x2), uvs);
        mesh.insert_attribute(MeshVertexAttribute::new("texindex", 2, VertexFormat::Uint32), texture_indices);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }
}

#[test]
fn mesher_test() {
    let mut mesher = MeshBuilder::new();

}