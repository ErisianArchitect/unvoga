#![allow(unused)]
use bevy::{math::{Vec2, Vec3}, prelude::Mesh, render::{mesh::{MeshVertexAttribute, PrimitiveTopology}, render_asset::RenderAssetUsages, render_resource::VertexFormat}};

use crate::prelude::{Orientation, Rotation};

use super::voxelmesh::MeshData;

#[derive(Debug, Default, Clone)]
pub struct MeshBuilder {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub texindices: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build<F: FnOnce(&mut Self)>(build: F) -> Self {
        let mut builder = Self::new();
        build(&mut builder);
        builder
    }

    pub fn build_mesh<F: FnOnce(&mut MeshBuilder)>(mesh: &mut Mesh, build: F) {
        let mut builder = Self::new();
        build(&mut builder);
        builder.push_to_mesh(mesh);
    }

    pub fn create_mesh<F: FnOnce(&mut MeshBuilder)>(render_asset_usages: RenderAssetUsages, build: F) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, render_asset_usages);
        Self::build_mesh(&mut mesh, build);
        mesh
    }

    /// This method assumes that your mesh_data is valid.
    /// That means that it has the same number of vertices, normals, uvs, and texindices.
    pub fn push_mesh_data(&mut self, mesh_data: &MeshData) {
        let start_index = self.vertices.len() as u32;
        self.vertices.extend(mesh_data.vertices.iter().cloned());
        self.normals.extend(mesh_data.normals.iter().cloned());
        self.uvs.extend(mesh_data.uvs.iter().cloned());
        self.texindices.extend(mesh_data.texindices.iter().cloned());
        self.indices.extend(mesh_data.indices.iter().map(|&i| start_index + i));
    }

    pub fn push_oriented_mesh_data(&mut self, mesh_data: &MeshData, orientation: Orientation) {
        let invert_indices = orientation.flip.x() ^ orientation.flip.y() ^ orientation.flip.z();
        let start_index = self.vertices.len() as u32;
        self.uvs.extend(mesh_data.uvs.iter().cloned());
        self.texindices.extend(mesh_data.texindices.iter().cloned());
        self.vertices.extend(mesh_data.vertices.iter().cloned().map(|vert| orientation.transform(vert)));
        self.normals.extend(mesh_data.normals.iter().cloned().map(|norm| orientation.transform(norm)));
        if invert_indices {
            self.indices.extend(mesh_data.indices.iter().rev().map(|&i| start_index + i));
        } else {
            self.indices.extend(mesh_data.indices.iter().map(|&i| start_index + i));
        }
    }

    pub fn push_to_mesh(self, mesh: &mut Mesh) {
        mesh.insert_attribute(MeshVertexAttribute::new("position", 0, VertexFormat::Float32x3), self.vertices);
        mesh.insert_attribute(MeshVertexAttribute::new("uv", 1, VertexFormat::Float32x2), self.uvs);
        mesh.insert_attribute(MeshVertexAttribute::new("normal", 2, VertexFormat::Float32x3), self.normals);
        mesh.insert_attribute(MeshVertexAttribute::new("texindex", 3, VertexFormat::Uint32), self.texindices);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(self.indices));
    }

    pub fn mesh_data(self) -> MeshData {
        self.into()
    }
}

impl Into<MeshData> for MeshBuilder {
    fn into(self) -> MeshData {
        MeshData {
            vertices: self.vertices,
            normals: self.normals,
            uvs: self.uvs,
            texindices: self.texindices,
            indices: self.indices,
        }
    }
}

impl Into<Mesh> for MeshBuilder {
    fn into(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        self.push_to_mesh(&mut mesh);
        mesh
    }
}