use crate::{core::voxel::rendering::voxelmesh::MeshData, prelude::Faces};

pub struct ModelInfo {
    faces: Option<Box<Faces<Option<String>>>>
}

pub struct ModelData {
    faces: Option<Box<Faces<Option<MeshData>>>>,
    extra: Option<Box<MeshData>>,
}