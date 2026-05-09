//! `SolidBlock` — the workhorse cube block. Six prebaked faces, one texture
//! index per face. Was bin-private to `sandbox`; promoted so `cityvox` (and
//! anything else) can register textured cubes without copy-pasting it.

use std::sync::LazyLock;

use bevy::math::{vec2, vec3, Vec3};

use crate::core::voxel::block::Block;
use crate::core::voxel::blocks::Id;
use crate::core::voxel::blockstate::BlockState;
use crate::core::voxel::coord::Coord;
use crate::core::voxel::direction::Direction;
use crate::core::voxel::faces::Faces;
use crate::core::voxel::level_of_detail::LOD;
use crate::core::voxel::world::occlusion::Occlusion;
use crate::core::voxel::rendering::meshbuilder::MeshBuilder;
use crate::core::voxel::rendering::voxelmesh::MeshData;
use crate::core::voxel::world::VoxelWorld;
use crate::core::math::orientation::Orientation;
use crate::core::math::rotation::Rotation;

pub struct SolidBlock {
    mesh_data: Faces<MeshData>,
    name: String,
    default_state: BlockState,
}

impl SolidBlock {
    pub fn single<S: AsRef<str>>(name: S, default_state: BlockState, texture_index: u32) -> Self {
        Self::new(
            name,
            default_state,
            Faces::new(
                texture_index,
                texture_index,
                texture_index,
                texture_index,
                texture_index,
                texture_index,
            ),
        )
    }

    pub fn vertical_block<S: AsRef<str>>(
        name: S,
        default_state: BlockState,
        vertical_texture_index: u32,
        side_texture_index: u32,
    ) -> Self {
        Self::new(
            name,
            default_state,
            Faces::new(
                side_texture_index,
                vertical_texture_index,
                side_texture_index,
                side_texture_index,
                vertical_texture_index,
                side_texture_index,
            ),
        )
    }

    pub fn new<S: AsRef<str>>(name: S, default_state: BlockState, texindices: Faces<u32>) -> Self {
        static POS_Y_MESH: LazyLock<MeshData> = LazyLock::new(|| MeshData {
            vertices: vec![
                vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, -0.5),
                vec3(-0.5, 0.5, 0.5),  vec3(0.5, 0.5, 0.5),
            ],
            normals: vec![Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y],
            uvs: vec![
                vec2(0.0, 0.0), vec2(1.0, 0.0),
                vec2(0.0, 1.0), vec2(1.0, 1.0),
            ],
            texindices: vec![0, 0, 0, 0],
            indices: vec![0, 2, 1, 1, 2, 3],
        });
        let pos_x_mesh = POS_Y_MESH
            .clone()
            .map_orientation(Rotation::new(Direction::PosX, 0).into())
            .map_texindices(texindices.pos_x);
        let pos_z_mesh = POS_Y_MESH
            .clone()
            .map_orientation(Rotation::new(Direction::PosZ, 0).into())
            .map_texindices(texindices.pos_z);
        let neg_x_mesh = POS_Y_MESH
            .clone()
            .map_orientation(Rotation::new(Direction::NegX, 0).into())
            .map_texindices(texindices.neg_x);
        let neg_y_mesh = POS_Y_MESH
            .clone()
            .map_orientation(Rotation::new(Direction::NegY, 0).into())
            .map_texindices(texindices.neg_y);
        let neg_z_mesh = POS_Y_MESH
            .clone()
            .map_orientation(Rotation::new(Direction::NegZ, 0).into())
            .map_texindices(texindices.neg_z);
        Self {
            mesh_data: Faces {
                pos_x: pos_x_mesh,
                pos_y: POS_Y_MESH.clone().map_texindices(texindices.pos_y),
                pos_z: pos_z_mesh,
                neg_x: neg_x_mesh,
                neg_y: neg_y_mesh,
                neg_z: neg_z_mesh,
            },
            name: name.as_ref().to_owned(),
            default_state,
        }
    }
}

impl Block for SolidBlock {
    fn name(&self) -> &str {
        &self.name
    }

    fn default_state(&self) -> BlockState {
        self.default_state.clone()
    }

    fn push_mesh(
        &self,
        mesh_builder: &mut MeshBuilder,
        _level_of_detail: LOD,
        _world: &VoxelWorld,
        _coord: Coord,
        _state: Id,
        occlusion: Occlusion,
        orientation: Orientation,
    ) {
        Direction::iter().for_each(|dir| {
            if occlusion.visible(dir) {
                let _src_face = orientation.source_face(dir);
                mesh_builder.push_mesh_data(self.mesh_data.face(dir));
            }
        });
    }
}
