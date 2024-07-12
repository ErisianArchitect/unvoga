pub mod voxelmaterial;
pub mod voxelmesh;
pub mod meshbuilder;

use bevy::{asset::Handle, render::mesh::Mesh};

use super::faces::Faces;

pub type MeshFaces = Faces<Handle<Mesh>>;