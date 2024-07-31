use std::{cell::RefCell, path::{Path, PathBuf}, sync::LazyLock};

use bevy::prelude::Image;
use bevy_egui::egui::mutex::Mutex;
use hashbrown::HashMap;
use crate::core::util::texture_array::{create_texture_array, create_texture_array_from_paths, BuildTextureArrayError};

struct RegEntry {
    name: String,
    path: PathBuf,
    texture_index: u32,
}

struct TextureRegistry {
    entries: Vec<RegEntry>,
    // u16 because there's a limit to how many layers a texture array can have (I think).
    lookup: HashMap<String, u16>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, path: PathBuf) -> u32 {
        if let Some(&entry_id) = self.lookup.get(&name) {
            if self.entries[entry_id as usize].path != path {
                panic!("Tried to already registered \"{name}\" with different path.");
            }
            self.entries[entry_id as usize].texture_index
        } else {
            let index = self.entries.len() as u16;
            self.lookup.insert(name.clone(), index);
            self.entries.push(RegEntry { name: name, path: path, texture_index: index as u32 });
            index as u32
        }
    }

    pub fn get_texture_index<S: AsRef<str>>(&self, name: S) -> u32 {
        if let Some(&entry) = self.lookup.get(name.as_ref()) {
            entry as u32
        } else {
            panic!("Registry entry not found.");
        }
    }

    pub fn build_texture_array(&self, width: u32, height: u32) -> Result<Image, BuildTextureArrayError> {
        create_texture_array_from_paths(width, height, self.entries.iter().map(|entry| entry.path.clone()).collect())
    }
}

static REGISTRY: LazyLock<Mutex<TextureRegistry>> = LazyLock::new(|| Mutex::new(TextureRegistry::new()));

pub fn register<S: AsRef<str>, P: AsRef<Path>>(name: S, path: P) -> u32 {
    let mut registry = REGISTRY.lock();
    registry.register(name.as_ref().to_owned(), path.as_ref().to_owned())
}

pub fn get_texture_index<S: AsRef<str>>(name: S) -> u32 {
    let mut registry = REGISTRY.lock();
    registry.get_texture_index(name)
}

pub fn build_texture_array(width: u32, height: u32) -> Result<Image, BuildTextureArrayError> {
    let mut registry = REGISTRY.lock();
    registry.build_texture_array(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_test() {
        register("sand", "./assets/debug/textures/blocks/sand.png");
        println!("Sand: {}", get_texture_index("sand"));
        let image = build_texture_array(256, 256).expect("Failed");
    }
}