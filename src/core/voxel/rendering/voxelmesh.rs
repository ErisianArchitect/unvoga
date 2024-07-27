#![allow(unused)]
use bevy::{math::{Vec2, Vec3}, prelude::Mesh, render::{mesh::{MeshVertexAttribute, PrimitiveTopology}, render_asset::RenderAssetUsages, render_resource::VertexFormat}};

use crate::{core::voxel::faces::Faces, prelude::Orientation};

use super::meshbuilder::MeshBuilder;

pub const POSITION_ATTRIB: MeshVertexAttribute = MeshVertexAttribute::new("position", 0, VertexFormat::Float32x3);
pub const NORMAL_ATTRIB: MeshVertexAttribute = MeshVertexAttribute::new("normal", 2, VertexFormat::Float32x3);
pub const UV_ATTRIB: MeshVertexAttribute = MeshVertexAttribute::new("uv", 1, VertexFormat::Float32x2);
pub const TEXINDEX_ATTRIB: MeshVertexAttribute = MeshVertexAttribute::new("texindex", 3, VertexFormat::Uint32);

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

    pub fn to_mesh(self, render_asset_usages: RenderAssetUsages) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, render_asset_usages);
        mesh.insert_attribute(POSITION_ATTRIB.clone(), self.vertices);
        mesh.insert_attribute(UV_ATTRIB.clone(), self.uvs);
        mesh.insert_attribute(NORMAL_ATTRIB.clone(), self.normals);
        mesh.insert_attribute(TEXINDEX_ATTRIB.clone(), self.texindices);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(self.indices));
        mesh
    }

    pub fn recalculate_normals(mut self) -> Self {
        (0..self.indices.len()).step_by(3)
            .map(|i| [self.indices[i] as usize, self.indices[i + 1] as usize, self.indices[i + 2] as usize])
            .for_each(|tri| {
                let verts = [
                    self.vertices[tri[0]],
                    self.vertices[tri[1]],
                    self.vertices[tri[2]]
                ];
                let norm = crate::core::math::vector::calculate_tri_normal(&verts);
                self.normals[tri[0]] = norm;
                self.normals[tri[1]] = norm;
                self.normals[tri[2]] = norm;
            });
        self
    }
}

pub struct VoxelMesh {
    pub faces: Faces<MeshData>,
    pub unoriented: MeshData,
}

impl Into<Mesh> for MeshData {
    /// This will create a mesh with [RenderAssetUsages::all()], if you don't want that, you should
    /// use [MeshData::to_mesh].
    fn into(self) -> Mesh {
        self.to_mesh(RenderAssetUsages::all())
    }
}