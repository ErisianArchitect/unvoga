use bevy::prelude::*;

#[derive(Resource)]
pub struct VoxelWorldResources {
    texture_array: Handle<Image>,
}

impl VoxelWorldResources {
    pub fn new(
        texture_array: Handle<Image>,
    ) -> Self {
        Self {
            texture_array,
        }
    }

    pub fn texture_array(&self) -> Handle<Image> {
        self.texture_array.clone()
    }
}

