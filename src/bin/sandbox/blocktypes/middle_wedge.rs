use std::sync::LazyLock;

use unvoga::{blockstate, core::util::modelimporter::{read_model_data, ModelData}, prelude::{Block, Direction, Occluder, OcclusionRect, OcclusionShape, OcclusionShape2x2, OcclusionShape4x4, Orientation, StateValue}};

pub struct MiddleWedge;

impl Block for MiddleWedge {
    fn name(&self) -> &str {
        "middle_wedge"
    }

    fn default_state(&self) -> unvoga::core::voxel::blockstate::BlockState {
        blockstate!(middle_wedge)
    }

    fn occluder(&self, world: &unvoga::core::voxel::world::VoxelWorld, state: unvoga::prelude::Id) -> &unvoga::prelude::Occluder {
        const OCCLUDER: Occluder = Occluder {
            neg_x: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [0, 0],
                [1, 0]
            ])),
            pos_x: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [0, 0],
                [0, 1]
            ])),
            neg_z: OcclusionShape::S4x4(OcclusionShape4x4::from_matrix([
                [0, 0, 0, 0],
                [1, 1, 1, 1],
                [1, 1, 1, 1],
                [1, 1, 1, 1]
            ])),
            pos_z: OcclusionShape::S4x4(OcclusionShape4x4::from_matrix([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 1, 1],
            ])),
            pos_y: OcclusionShape::Empty,
            neg_y: OcclusionShape::Full
        };
        &OCCLUDER
    }

    fn occludee(&self, world: &unvoga::core::voxel::world::VoxelWorld, state: unvoga::prelude::Id) -> &Occluder {
        const OCCLUDEE: Occluder = Occluder {
            neg_x: OcclusionShape::S4x4(OcclusionShape4x4::from_matrix([
                [0, 0, 0, 0],
                [1, 1, 0, 0],
                [1, 1, 1, 1],
                [1, 1, 1, 1]
            ])),
            pos_x: OcclusionShape::S4x4(OcclusionShape4x4::from_matrix([
                [0, 0, 0, 0],
                [0, 0, 1, 1],
                [1, 1, 1, 1],
                [1, 1, 1, 1]
            ])),
            neg_z: OcclusionShape::S4x4(OcclusionShape4x4::from_matrix([
                [0, 0, 0, 0],
                [1, 1, 1, 1],
                [1, 1, 1, 1],
                [1, 1, 1, 1]
            ])),
            pos_z: OcclusionShape::S4x4(OcclusionShape4x4::from_matrix([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 1, 1],
            ])),
            pos_y: OcclusionShape::Empty,
            neg_y: OcclusionShape::Full
        };
        &OCCLUDEE
    }

    fn orientation(&self, world: &unvoga::core::voxel::world::VoxelWorld, coord: unvoga::prelude::Coord, state: unvoga::prelude::Id) -> unvoga::prelude::Orientation {
        if let StateValue::Orientation(orientation) = state["orientation"] {
            orientation
        } else {
            Orientation::UNORIENTED
        }
    }

    fn push_mesh(&self, mesh_builder: &mut unvoga::core::voxel::rendering::meshbuilder::MeshBuilder, world: &unvoga::core::voxel::world::VoxelWorld, coord: unvoga::prelude::Coord, state: unvoga::prelude::Id, occlusion: unvoga::prelude::Occlusion, orientation: Orientation) {
        static MODEL: LazyLock<ModelData> = LazyLock::new(|| {
            read_model_data("./assets/debug/models/middle_wedge.json", None).expect("Failed to read model for middle_wedge.")
        });
        if occlusion.neg_x() {
            let src_face = orientation.source_face(Direction::NegX);
            mesh_builder.push_mesh_data(MODEL.face(src_face));
        }
        if occlusion.pos_x() {
            let src_face = orientation.source_face(Direction::PosX);
            mesh_builder.push_mesh_data(MODEL.face(src_face));
        }
        if occlusion.neg_z() {
            let src_face = orientation.source_face(Direction::NegZ);
            mesh_builder.push_mesh_data(MODEL.face(src_face));
        }
        if occlusion.pos_z() {
            let src_face = orientation.source_face(Direction::PosZ);
            mesh_builder.push_mesh_data(MODEL.face(src_face));
        }
        if occlusion.neg_y() {
            let src_face = orientation.source_face(Direction::NegY);
            mesh_builder.push_mesh_data(MODEL.face(src_face));
        }
        if occlusion.pos_y() {
            let src_face = orientation.source_face(Direction::PosY);
            mesh_builder.push_mesh_data(MODEL.face(src_face));
        }
    }
}