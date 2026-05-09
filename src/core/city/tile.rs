use serde::{Deserialize, Serialize};

use super::projection::{GeoPoint, LocalPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CityTileId {
    pub x: i32,
    pub z: i32,
    pub lod: u8,
}

impl CityTileId {
    pub const fn new(x: i32, z: i32, lod: u8) -> Self {
        Self { x, z, lod }
    }

    pub fn from_local(point: LocalPoint, tile_size_m: f64, lod: u8) -> Self {
        Self {
            x: (point.x / tile_size_m).floor() as i32,
            z: (point.z / tile_size_m).floor() as i32,
            lod,
        }
    }

    pub fn min_local(self, tile_size_m: f64) -> LocalPoint {
        LocalPoint::new(self.x as f64 * tile_size_m, self.z as f64 * tile_size_m)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceTag {
    Procedural,
    Overture,
    OpenStreetMap,
    GovernmentOpenData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadClass {
    Motorway,
    Arterial,
    Collector,
    Local,
    Service,
    Footway,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub id: String,
    pub class: RoadClass,
    pub centerline: Vec<LocalPoint>,
    pub width_m: f32,
    pub lanes: u8,
    pub source: SourceTag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub footprint: Vec<LocalPoint>,
    pub min_height_m: f32,
    pub height_m: f32,
    pub levels: Option<u16>,
    pub source: SourceTag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub category: String,
    pub point: LocalPoint,
    pub source: SourceTag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CityTile {
    pub id: CityTileId,
    pub origin: GeoPoint,
    pub tile_size_m: f64,
    pub roads: Vec<Road>,
    pub buildings: Vec<Building>,
    pub places: Vec<Place>,
}

impl CityTile {
    pub fn new(id: CityTileId, origin: GeoPoint, tile_size_m: f64) -> Self {
        Self {
            id,
            origin,
            tile_size_m,
            roads: Vec::new(),
            buildings: Vec::new(),
            places: Vec::new(),
        }
    }

    pub fn min_local(&self) -> LocalPoint {
        self.id.min_local(self.tile_size_m)
    }

    pub fn max_local(&self) -> LocalPoint {
        let min = self.min_local();
        LocalPoint::new(min.x + self.tile_size_m, min.z + self.tile_size_m)
    }

    pub fn feature_count(&self) -> usize {
        self.roads.len() + self.buildings.len() + self.places.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_id_uses_floor_for_negative_coordinates() {
        assert_eq!(
            CityTileId::from_local(LocalPoint::new(-1.0, -513.0), 512.0, 0),
            CityTileId::new(-1, -2, 0)
        );
    }
}
