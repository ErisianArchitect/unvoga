use bevy::math::{Vec2, Vec3};

use crate::core::voxel::faces::Faces;

pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub texindices: Vec<u32>,
    pub indices: Vec<u32>,
}

pub struct VoxelMesh {
    pub faces: Faces<MeshData>,
    pub unoriented: MeshData,
}