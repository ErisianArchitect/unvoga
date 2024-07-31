use std::path::{Path, PathBuf};

use crate::core::util::textureregistry;
use crate::core::voxel::rendering::meshbuilder::MeshBuilder;
use crate::prelude::Direction;
use crate::{core::voxel::rendering::voxelmesh::MeshData, prelude::Faces};
use crate::core::error::*;
use bevy::math::{Vec2, Vec3};
use hashbrown::HashMap;
use itertools::Itertools;
use serde::{
    Serialize,
    Deserialize,
};
use serde_json::{
    json,
    Value,
};

// pub struct ModelInfo {
//     faces: Option<Box<Faces<Option<String>>>>
// }

#[derive(Debug, Default, Clone)]
pub struct ModelData {
    faces: Option<Box<Faces<Option<Box<MeshData>>>>>,
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

#[derive(Debug, Default, Clone)]
pub struct ModelMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

impl ModelMesh {
    pub fn combine(&mut self, other: &ModelMesh) {
        self.push_data(&other.positions, &other.normals, &other.uvs, &other.indices);
    }

    pub fn push_data(
        &mut self, 
        positions: &[Vec3],
        normals: &[Vec3],
        uvs: &[Vec2],
        indices: &[u32]
    ) {
        let vert_index = self.positions.len() as u32;
        self.positions.extend(positions);
        self.normals.extend(normals);
        self.uvs.extend(uvs);
        self.indices.extend(indices.iter().cloned().map(|index| vert_index + index));
    }

    pub fn to_meshdata(self, texture_index: u32) -> MeshData {
        let texindices = (0..self.positions.len()).map(|_| texture_index).collect_vec();
        MeshData {
            vertices: self.positions,
            normals: self.normals,
            uvs: self.uvs,
            indices: self.indices,
            texindices,
        }
    }
}

pub fn extract_meshes_from_gltf<P: AsRef<Path>>(path: P) -> Result<HashMap<String, ModelMesh>> {
    let mut meshes = HashMap::new();
    let (model, buffers, _) = gltf::import(path)?;
    for mesh in model.meshes() {
        let mut model_mesh = ModelMesh::default();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| {
                Some(&buffers[buffer.index()])
            });
            let Some(positions) = reader.read_positions() else {
                return Err(Error::MeshExtractionError);
            };
            let Some(normals) = reader.read_normals() else {
                return Err(Error::MeshExtractionError);
            };
            let Some(uvs) = reader.read_tex_coords(0) else {
                return Err(Error::MeshExtractionError);
            };
            let Some(indices) = reader.read_indices() else {
                return Err(Error::MeshExtractionError);
            };
            let positions = positions.map(Vec3::from).collect_vec();

            let normals = normals.map(Vec3::from).collect_vec();
            let uvs = uvs.into_f32().map(Vec2::from).collect_vec();
            let indices = indices.into_u32().collect_vec();
            model_mesh.push_data(&positions, &normals, &uvs, &indices);
        }
        let Some(name) = mesh.name() else {
            return Err(Error::MeshExtractionError);
        };
        meshes.insert(name.to_owned(), model_mesh);
    }
    Ok(meshes)
}

pub fn read_model_json<P: AsRef<Path>>(path: P) -> Result<ModelJson> {
    use std::fs::File;
    use std::io::{
        Read, BufReader
    };
    let mut reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}

/// This function might register textures to the texture registry.
pub fn read_model_data<P: AsRef<Path>>(path: P, textures: Option<HashMap<String, String>>) -> Result<ModelData> {
    let model_json = read_model_json(path)?;
    let mut texture_map = HashMap::<String, u32>::new();
    // collect texture indices into texture_map
    model_json.textures.iter().for_each(|(key, modtex)| {
        if let Some(textures) = &textures {
            if let Some(input) = textures.get(key) {
                let index = textureregistry::get_texture_index(input);
                texture_map.insert(key.to_owned(), index);
                // Early return so we don't fall through to the statements below.
                return;
            }
        }
        let index = textureregistry::register(&modtex.name, &modtex.path);
        texture_map.insert(key.to_owned(), index);
    });
    let meshes = extract_meshes_from_gltf(model_json.model)?;
    let mut model_data = ModelData::default();
    model_data.extra = if let Some(extra) = model_json.mesh_data.extra {
        // name is mesh name
        // texture is key to texture_map
        let mut build = MeshBuilder::new();
        extra.into_iter().try_for_each(|MeshTexture { name, texture }| {
            let Some(mesh) = meshes.get(&name) else {  
                return Err(Error::MeshExtractionError);
            };
            let Some(&texindex) = texture_map.get(&texture) else {
                return Err(Error::MeshExtractionError);
            };
            let meshdata = mesh.clone().to_meshdata(texindex);
            build.push_mesh_data(&meshdata);
            Result::Ok(())
        })?;
        Some(Box::new(build.to_mesh_data()))
    } else {
        None
    };
    macro_rules! model_face {
        ($face:ident) => {
            if let Some($face) = model_json.mesh_data.$face {
                let mut build = MeshBuilder::new();
                $face.into_iter().try_for_each(|MeshTexture { name, texture }| {
                    let Some(mesh) = meshes.get(&name) else {  
                        return Err(Error::MeshExtractionError);
                    };
                    let Some(&texindex) = texture_map.get(&texture) else {
                        return Err(Error::MeshExtractionError);
                    };
                    let meshdata = mesh.clone().to_meshdata(texindex);
                    build.push_mesh_data(&meshdata);
                    Result::Ok(())
                })?;
                let faces = model_data.faces.get_or_insert_with(|| Box::new(Faces::new(None, None, None, None, None, None)));
                faces.$face = Some(Box::new(build.to_mesh_data()));
            }
        };
    }
    model_face!(pos_x);
    model_face!(pos_y);
    model_face!(pos_z);
    model_face!(neg_x);
    model_face!(neg_y);
    model_face!(neg_z);
    Ok(model_data)
}

#[cfg(test)]
mod testing_sandbox {
    use super::*;
    #[test]
    fn sandbox() {
        let mid_wedge = "./assets/debug/models/middle_wedge.json";
        let model = read_model_data(mid_wedge, None).expect("Failed to read the model");
    }
}

/* ModelInfo JSON Prototype
{
    "top": {

    }
}
*/