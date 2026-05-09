#![allow(unused)]
use bevy::{prelude::*, render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer}};

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct VoxelMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
    #[uniform(2)]
    pub light_level: f32,
    #[storage(3, read_only)]
    pub lightmap: Handle<ShaderStorageBuffer>,
    #[storage(4, read_only)]
    pub lightmap_pad_pos_x: Handle<ShaderStorageBuffer>,
    #[storage(5, read_only)]
    pub lightmap_pad_neg_x: Handle<ShaderStorageBuffer>,
    #[storage(6, read_only)]
    pub lightmap_pad_pos_y: Handle<ShaderStorageBuffer>,
    #[storage(7, read_only)]
    pub lightmap_pad_neg_y: Handle<ShaderStorageBuffer>,
    #[storage(8, read_only)]
    pub lightmap_pad_pos_z: Handle<ShaderStorageBuffer>,
    #[storage(9, read_only)]
    pub lightmap_pad_neg_z: Handle<ShaderStorageBuffer>,
}

pub const MIN_LIGHT_LEVEL: f32 = 0.025;
pub const MAX_LIGHT_LEVEL: f32 = 1.0;

fn make_buffer(buffers: &mut Assets<ShaderStorageBuffer>, len: usize) -> Handle<ShaderStorageBuffer> {
    let data: Vec<f32> = (0..len).map(|_| 1.0).collect();
    buffers.add(ShaderStorageBuffer::from(data))
}

impl VoxelMaterial {
    pub fn new(array_texture: Handle<Image>, buffers: &mut Assets<ShaderStorageBuffer>) -> Self {
        Self {
            array_texture,
            light_level: MIN_LIGHT_LEVEL,
            lightmap: make_buffer(buffers, 4096),
            lightmap_pad_pos_x: make_buffer(buffers, 256),
            lightmap_pad_neg_x: make_buffer(buffers, 256),
            lightmap_pad_pos_y: make_buffer(buffers, 256),
            lightmap_pad_neg_y: make_buffer(buffers, 256),
            lightmap_pad_pos_z: make_buffer(buffers, 256),
            lightmap_pad_neg_z: make_buffer(buffers, 256),
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
}
