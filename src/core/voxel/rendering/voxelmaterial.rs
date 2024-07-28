#![allow(unused)]
use bevy::{prelude::*, render::render_resource::AsBindGroup};

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct VoxelMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
    #[uniform(2)]
    pub light_level: f32,
    #[storage(3, read_only)]
    pub lightmap: Vec<f32>,
    #[storage(4, read_only)]
    pub lightmap_pad_pos_x: Vec<f32>,
    #[storage(5, read_only)]
    pub lightmap_pad_neg_x: Vec<f32>,
    #[storage(6, read_only)]
    pub lightmap_pad_pos_y: Vec<f32>,
    #[storage(7, read_only)]
    pub lightmap_pad_neg_y: Vec<f32>,
    #[storage(8, read_only)]
    pub lightmap_pad_pos_z: Vec<f32>,
    #[storage(9, read_only)]
    pub lightmap_pad_neg_z: Vec<f32>,
}

pub const MIN_LIGHT_LEVEL: f32 = 0.025;
pub const MAX_LIGHT_LEVEL: f32 = 1.0;

impl VoxelMaterial {
    pub fn new(array_texture: Handle<Image>) -> Self {
        Self {
            array_texture,
            light_level: MIN_LIGHT_LEVEL,
            // lightmap: vec![],
            // lightmap_pad_pos_x: vec![],
            // lightmap_pad_neg_x: vec![],
            // lightmap_pad_pos_y: vec![],
            // lightmap_pad_neg_y: vec![],
            // lightmap_pad_pos_z: vec![],
            // lightmap_pad_neg_z: vec![],
            lightmap: (0..4096).map(|_| 1.0).collect(),
            lightmap_pad_pos_x: (0..256).map(|_| 1.0).collect(),
            lightmap_pad_neg_x: (0..256).map(|_| 1.0).collect(),
            lightmap_pad_pos_y: (0..256).map(|_| 1.0).collect(),
            lightmap_pad_neg_y: (0..256).map(|_| 1.0).collect(),
            lightmap_pad_pos_z: (0..256).map(|_| 1.0).collect(),
            lightmap_pad_neg_z: (0..256).map(|_| 1.0).collect(),
        }
    }
}

impl Material for VoxelMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/voxel/voxel.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/voxel/voxel.wgsl".into()
    }

    // fn alpha_mode(&self) -> AlphaMode {
    //     AlphaMode::
    // }
}