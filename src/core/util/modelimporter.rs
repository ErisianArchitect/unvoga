use std::path::{Path, PathBuf};

use crate::prelude::Direction;
use crate::{core::voxel::rendering::voxelmesh::MeshData, prelude::Faces};
use crate::core::error::*;
use hashbrown::HashMap;
use serde::{
    Serialize,
    Deserialize,
};
use serde_json::{
    json,
    Value,
};

pub struct ModelInfo {
    faces: Option<Box<Faces<Option<String>>>>
}

pub struct ModelData {
    faces: Option<Box<Faces<Option<MeshData>>>>,
    extra: Option<Box<MeshData>>,
}

impl ModelData {
    /// This method will panic if the face is not present!
    /// I expect that you would know whether or not a face is present
    /// if you are using this ModelData, as I would assume that you imported it
    /// from a file that you created.
    pub fn face(&self, face: Direction) -> &MeshData {
        self.faces.as_ref().unwrap().face(face).as_ref().unwrap()
    }

    /// This method will panic if extra is not present.
    pub fn extra(&self) -> &MeshData {
        self.extra.as_ref().unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelJson {
    pub model: PathBuf,
    pub textures: HashMap<String, ModelTexture>,
    pub mesh_data: ModelJsonMeshData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelJsonMeshData {
    pub extra: Option<Vec<MeshTexture>>,
    pub pos_x: Option<Vec<MeshTexture>>,
    pub pos_y: Option<Vec<MeshTexture>>,
    pub pos_z: Option<Vec<MeshTexture>>,
    pub neg_x: Option<Vec<MeshTexture>>,
    pub neg_y: Option<Vec<MeshTexture>>,
    pub neg_z: Option<Vec<MeshTexture>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTexture {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTexture {
    pub name: String,
    pub texture: String,
}

pub fn read_model_json<P: AsRef<Path>>(path: P) -> Result<ModelJson> {
    use std::fs::File;
    use std::io::{
        Read, BufReader
    };
    let mut reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}

#[cfg(test)]
mod testing_sandbox {
    use super::*;
    #[test]
    fn sandbox() {
        let mid_wedge = "./assets/debug/models/middle_wedge.json";
        let data = read_model_json(mid_wedge).expect("Failed to read JSON.");
        println!("Model: {}", data.model.display());
        println!("Textures: {:?}", data.textures);
        println!("Mesh Data: {:?}", data.mesh_data);
    }
}

/* ModelInfo JSON Prototype
{
    "top": {

    }
}
*/