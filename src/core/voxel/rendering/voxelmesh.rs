#![allow(unused)]
use bevy::{math::{Vec2, Vec3}, prelude::Mesh, render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages}};

use crate::{core::voxel::faces::Faces, prelude::Orientation};

use super::meshbuilder::MeshBuilder;

#[derive(Debug, Default, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub texindices: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub const fn new(
        vertices: Vec<Vec3>,
        normals: Vec<Vec3>,
        uvs: Vec<Vec2>,
        texindices: Vec<u32>,
        indices: Vec<u32>
    ) -> Self {
        Self {
            vertices,
            normals,
            uvs,
            texindices,
            indices
        }
    }

    pub fn map_orientation(mut self, orientation: Orientation) -> Self {
        self.vertices.iter_mut().for_each(|vert| *vert = orientation.transform(*vert));
        self.normals.iter_mut().for_each(|norm| *norm = orientation.transform(*norm));
        let invert_indices = orientation.flip.xor();
        if invert_indices {
            self.indices.reverse();
        }
        self
    }

    pub fn map_texindices(mut self, index: u32) -> Self {
        self.texindices.iter_mut().for_each(|i| *i = index);
        self
    }
}

pub struct VoxelMesh {
    pub faces: Faces<MeshData>,
    pub unoriented: MeshData,
}

impl Into<Mesh> for MeshData {
    fn into(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        MeshBuilder::build_mesh(&mut mesh, |build| build.push_mesh_data(&self));
        mesh
    }
}