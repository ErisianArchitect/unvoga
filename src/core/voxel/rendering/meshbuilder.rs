#![allow(unused)]
use bevy::{math::{Vec2, Vec3}, prelude::Mesh, render::{mesh::{Indices, MeshVertexAttribute, PrimitiveTopology}, render_asset::RenderAssetUsages, render_resource::VertexFormat}};

use crate::prelude::{Orientation, Rotation};

use super::voxelmesh::MeshData;

#[derive(Debug, Default, Clone)]
pub struct MeshBuilder {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub texindices: Vec<u32>,
    pub indices: Vec<u32>,
    pub offset: Option<Vec3>,
    pub orientation: Option<Orientation>,
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

    pub fn set_offset<C: Into<Vec3>>(&mut self, offset: C) {
        self.offset = Some(offset.into());
    }

    pub fn clear_offset(&mut self) {
        self.offset.take();
    }

    pub fn set_orientation<O: Into<Orientation>>(&mut self, orientation: O) {
        self.orientation = Some(orientation.into());
    }

    pub fn clear_orientation(&mut self) {
        self.orientation.take();
    }

    /// Push exact mesh data without transformations.
    pub fn push_exact_mesh_data(&mut self, mesh_data: &MeshData) {
        let start_index = self.vertices.len() as u32;
        self.vertices.extend(mesh_data.vertices.iter().cloned());
        self.normals.extend(mesh_data.normals.iter().cloned());
        self.uvs.extend(mesh_data.uvs.iter().cloned());
        self.texindices.extend(mesh_data.texindices.iter().cloned());
        self.indices.extend(mesh_data.indices.iter().map(|&i| start_index + i));
    }

    /// Push transformed mesh data.
    pub fn push_mesh_data(&mut self, mesh_data: &MeshData) {
        let offset = self.offset.unwrap_or_default();
        let orientation = self.orientation.unwrap_or_default();
        let start_index = self.vertices.len() as u32;
        self.uvs.extend(mesh_data.uvs.iter().cloned());
        self.texindices.extend(mesh_data.texindices.iter().cloned());
        self.vertices.extend(mesh_data.vertices.iter().cloned().map(|vert| orientation.transform(vert) + offset));
        self.normals.extend(mesh_data.normals.iter().cloned().map(|norm| orientation.transform(norm)));
        let invert_indices = orientation.flip.x() ^ orientation.flip.y() ^ orientation.flip.z();
        if invert_indices {
            self.indices.extend(mesh_data.indices.iter().rev().map(|&i| start_index + i));
        } else {
            self.indices.extend(mesh_data.indices.iter().map(|&i| start_index + i));
        }
    }

    pub fn push_to_mesh(self, mesh: &mut Mesh) {
        use super::voxelmesh::*;
        mesh.insert_attribute(POSITION_ATTRIB.clone(), self.vertices);
        mesh.insert_attribute(NORMAL_ATTRIB.clone(), self.normals);
        mesh.insert_attribute(UV_ATTRIB.clone(), self.uvs);
        mesh.insert_attribute(TEXINDEX_ATTRIB.clone(), self.texindices);
        mesh.insert_indices(Indices::U32(self.indices));
    }

    pub fn to_mesh_data(self) -> MeshData {
        MeshData {
            vertices: self.vertices,
            normals: self.normals,
            uvs: self.uvs,
            texindices: self.texindices,
            indices: self.indices,
        }
    }

    pub fn to_mesh(self, render_asset_usages: RenderAssetUsages) -> Mesh {
        use super::voxelmesh::*;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, render_asset_usages);
        self.push_to_mesh(&mut mesh);
        mesh
    }

    pub fn recalculate_normals(&mut self) {
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
    }
}

impl Into<MeshData> for MeshBuilder {
    fn into(self) -> MeshData {
        self.to_mesh_data()
    }
}

impl Into<Mesh> for MeshBuilder {
    /// This method creates a [Mesh] with [RenderAssetUsages::all()]. If you don't like that, use
    /// [MeshBuilder::to_mesh] instead.
    fn into(self) -> Mesh {
        self.to_mesh(RenderAssetUsages::all())
    }
}