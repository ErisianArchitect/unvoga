use bevy::math::{Vec2, Vec3};

use crate::core::voxel::faces::Faces;

pub struct MeshData {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    texindices: Vec<u32>,
    indices: Vec<u32>,
}

pub struct VoxelMesh {
    faces: Faces<MeshData>,
    unoriented: MeshData,
}