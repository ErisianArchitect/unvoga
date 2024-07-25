#![allow(unused)]
use std::path::Path;
use bevy::prelude::*;


#[derive(Debug, Resource, Clone, PartialEq, Eq, Hash)]
pub struct UnvogaSettings {
    // TODO
}

impl Default for UnvogaSettings {
    fn default() -> Self {
        Self {

        }
    }
}

impl UnvogaSettings {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        todo!()
    }
}