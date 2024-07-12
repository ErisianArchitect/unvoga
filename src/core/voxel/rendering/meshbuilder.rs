use bevy::math::{Vec2, Vec3};

use super::voxelmesh::MeshData;

pub struct MeshBuilder {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub texindices: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
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
}