pub mod voxel_material;
pub mod voxelmesh;

use bevy::{asset::Handle, render::mesh::Mesh};

use super::faces::Faces;

pub type MeshFaces = Faces<Handle<Mesh>>;