#![allow(unused)]
use bevy::{math::{Vec2, Vec3, Vec4}, prelude::Mesh, render::{mesh::{Indices, MeshVertexAttribute, PrimitiveTopology}, render_asset::RenderAssetUsages, render_resource::VertexFormat}};

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
        let start_index = self.vertices.len() as u32;
        let mut invert_indices = false;
        match (self.offset, self.orientation) {
            (None, Some(orientation)) if orientation != Orientation::UNORIENTED => {
                self.vertices.extend(mesh_data.vertices.iter().cloned().map(|vert| orientation.transform(vert)));
                self.normals.extend(mesh_data.normals.iter().cloned().map(|norm| orientation.transform(norm)));
                invert_indices = orientation.flip.x() ^ orientation.flip.y() ^ orientation.flip.z();
            }
            (Some(offset), Some(orientation)) if orientation != Orientation::UNORIENTED => {
                self.vertices.extend(mesh_data.vertices.iter().cloned().map(|vert| orientation.transform(vert) + offset));
                self.normals.extend(mesh_data.normals.iter().cloned().map(|norm| orientation.transform(norm)));
                invert_indices = orientation.flip.x() ^ orientation.flip.y() ^ orientation.flip.z();
            }
            (Some(offset), _) => {
                self.vertices.extend(mesh_data.vertices.iter().cloned().map(|vert| vert + offset));
                self.normals.extend(mesh_data.normals.iter().cloned());
            }
            (None, _) => {
                self.vertices.extend(mesh_data.vertices.iter().cloned());
                self.normals.extend(mesh_data.normals.iter().cloned());
            }
        }
        self.uvs.extend(mesh_data.uvs.iter().cloned());
        self.texindices.extend(mesh_data.texindices.iter().cloned());
        if invert_indices {
            self.indices.extend(mesh_data.indices.iter().rev().map(|&i| start_index + i));
        } else {
            self.indices.extend(mesh_data.indices.iter().map(|&i| start_index + i));
        }
    }

    pub fn push_iter<V, Vert, N, Norm, U, Uvs, T, Texi>(
        &mut self,
        vertices: Vert,
        normals: Norm,
        uvs: Uvs,
        texture_indices: Texi,
        indices: &[u32],
    )
    where
        V: Into<Vec3>,
        N: Into<Vec3>,
        U: Into<Vec2>,
        T: Into<u32>,
        Vert: IntoIterator<Item = V>,
        Norm: IntoIterator<Item = N>,
        Uvs: IntoIterator<Item = U>,
        Texi: IntoIterator<Item = T> {
            let start_index = self.vertices.len() as u32;
            let mut invert_indices = false;
            match (self.offset, self.orientation) {
                (None, Some(orientation)) if orientation != Orientation::UNORIENTED => {
                    self.vertices.extend(vertices.into_iter().map(V::into).map(|vert| orientation.transform(vert)));
                    self.normals.extend(normals.into_iter().map(N::into).map(|norm| orientation.transform(norm)));
                    invert_indices = orientation.flip.x() ^ orientation.flip.y() ^ orientation.flip.z();
                }
                (Some(offset), Some(orientation)) if orientation != Orientation::UNORIENTED => {
                    self.vertices.extend(vertices.into_iter().map(V::into).map(|vert| orientation.transform(vert) + offset));
                    self.normals.extend(normals.into_iter().map(N::into).map(|norm| orientation.transform(norm)));
                    invert_indices = orientation.flip.x() ^ orientation.flip.y() ^ orientation.flip.z();
                }
                (Some(offset), _) => {
                    self.vertices.extend(vertices.into_iter().map(V::into).map(|vert| vert + offset));
                    self.normals.extend(normals.into_iter().map(N::into));
                }
                (None, _) => {
                    self.vertices.extend(vertices.into_iter().map(V::into));
                    self.normals.extend(normals.into_iter().map(N::into));
                }
            }
            self.uvs.extend(uvs.into_iter().map(U::into));
            self.texindices.extend(texture_indices.into_iter().map(T::into));
            if invert_indices {
                self.indices.extend(indices.iter().cloned().rev().map(|i| start_index + i));
            } else {
                self.indices.extend(indices.iter().cloned().map(|i| start_index + i));
            }
        }

    pub fn push_exact_iter<V, Vert, N, Norm, U, Uvs, T, Texi>(
        &mut self,
        vertices: Vert,
        normals: Norm,
        uvs: Uvs,
        texture_indices: Texi,
        indices: &[u32],
    )
    where
        V: Into<Vec3>,
        N: Into<Vec3>,
        U: Into<Vec2>,
        T: Into<u32>,
        Vert: IntoIterator<Item = V>,
        Norm: IntoIterator<Item = N>,
        Texi: IntoIterator<Item = T>,
        Uvs: IntoIterator<Item = U> {
            let start_index = self.vertices.len() as u32;
            self.vertices.extend(vertices.into_iter().map(V::into));
            self.normals.extend(normals.into_iter().map(N::into));
            self.uvs.extend(uvs.into_iter().map(U::into));
            self.texindices.extend(texture_indices.into_iter().map(T::into));
            self.indices.extend(indices.iter().cloned().map(|i| start_index + i));
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

pub const COLOR_MESH_POSITION: MeshVertexAttribute = MeshVertexAttribute::new("position", 0, VertexFormat::Float32x3);
pub const COLOR_MESH_NORMAL: MeshVertexAttribute = MeshVertexAttribute::new("normal", 1, VertexFormat::Float32x3);
pub const COLOR_MESH_COLOR: MeshVertexAttribute =MeshVertexAttribute::new("color", 2, VertexFormat::Float32x4);

#[derive(Debug, Default, Clone)]
pub struct ColorMesh {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    colors: Vec<Vec4>,
    indices: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct ColorMeshBuilder {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    colors: Vec<Vec4>,
    indices: Vec<u32>,
}