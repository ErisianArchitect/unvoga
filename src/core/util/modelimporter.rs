use std::path::{Path, PathBuf};

use crate::{core::voxel::rendering::voxelmesh::MeshData, prelude::Faces};
use crate::core::error::*;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelJson {
    model: PathBuf,
    mesh_data: ModelJsonMeshData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelJsonMeshData {
    extra: Option<Vec<MeshTexture>>,
    pos_x: Option<Vec<MeshTexture>>,
    pos_y: Option<Vec<MeshTexture>>,
    pos_z: Option<Vec<MeshTexture>>,
    neg_x: Option<Vec<MeshTexture>>,
    neg_y: Option<Vec<MeshTexture>>,
    neg_z: Option<Vec<MeshTexture>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTexture {
    name: String,
    texture: String,
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
        println!("Mesh Data: {:?}", data.mesh_data);
    }
}

/* ModelInfo JSON Prototype
{
    "top": {

    }
}
*/