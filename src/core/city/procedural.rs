use super::projection::{GeoPoint, LocalPoint};
use super::tile::{Building, CityTile, CityTileId, Place, Road, RoadClass, SourceTag};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemoCityConfig {
    pub tile_size_m: f64,
    pub block_size_m: f64,
    pub street_width_m: f32,
    pub lot_margin_m: f64,
    pub min_building_height_m: f32,
    pub max_building_height_m: f32,
}

impl Default for DemoCityConfig {
    fn default() -> Self {
        Self {
            tile_size_m: 512.0,
            block_size_m: 64.0,
            street_width_m: 12.0,
            lot_margin_m: 8.0,
            min_building_height_m: 7.0,
            max_building_height_m: 75.0,
        }
    }
}

pub fn generate_demo_tile(id: CityTileId, origin: GeoPoint, config: DemoCityConfig) -> CityTile {
    let mut tile = CityTile::new(id, origin, config.tile_size_m);
    let min = id.min_local(config.tile_size_m);
    let max = LocalPoint::new(min.x + config.tile_size_m, min.z + config.tile_size_m);
    let blocks = (config.tile_size_m / config.block_size_m).ceil() as i32;

    for i in 0..=blocks {
        let offset = i as f64 * config.block_size_m;
        let x = (min.x + offset).min(max.x);
        let z = (min.z + offset).min(max.z);

        tile.roads.push(Road {
            id: format!("demo-road-x-{}-{}-{}", id.x, id.z, i),
            class: road_class_for_index(i),
            centerline: vec![LocalPoint::new(x, min.z), LocalPoint::new(x, max.z)],
            width_m: config.street_width_m,
            lanes: if i % 4 == 0 { 4 } else { 2 },
            source: SourceTag::Procedural,
        });

        tile.roads.push(Road {
            id: format!("demo-road-z-{}-{}-{}", id.x, id.z, i),
            class: road_class_for_index(i),
            centerline: vec![LocalPoint::new(min.x, z), LocalPoint::new(max.x, z)],
            width_m: config.street_width_m,
            lanes: if i % 4 == 0 { 4 } else { 2 },
            source: SourceTag::Procedural,
        });
    }

    for bz in 0..blocks {
        for bx in 0..blocks {
            let lot_min = LocalPoint::new(
                min.x + bx as f64 * config.block_size_m + config.lot_margin_m,
                min.z + bz as f64 * config.block_size_m + config.lot_margin_m,
            );
            let lot_max = LocalPoint::new(
                min.x + (bx + 1) as f64 * config.block_size_m - config.lot_margin_m,
                min.z + (bz + 1) as f64 * config.block_size_m - config.lot_margin_m,
            );
            if lot_min.x >= max.x
                || lot_min.z >= max.z
                || lot_max.x <= lot_min.x
                || lot_max.z <= lot_min.z
            {
                continue;
            }

            let hash = splitmix64(tile_seed(id, bx, bz));
            let height_t = (hash & 0xffff) as f32 / u16::MAX as f32;
            let height = config.min_building_height_m
                + (config.max_building_height_m - config.min_building_height_m)
                    * height_t.powf(1.8);
            let levels = (height / 3.2).round().max(1.0) as u16;
            tile.buildings.push(Building {
                id: format!("demo-building-{}-{}-{}-{}", id.x, id.z, bx, bz),
                footprint: vec![
                    lot_min,
                    LocalPoint::new(lot_max.x, lot_min.z),
                    lot_max,
                    LocalPoint::new(lot_min.x, lot_max.z),
                ],
                min_height_m: 0.0,
                height_m: height,
                levels: Some(levels),
                source: SourceTag::Procedural,
            });

            if hash % 7 == 0 {
                tile.places.push(Place {
                    id: format!("demo-place-{}-{}-{}-{}", id.x, id.z, bx, bz),
                    name: format!("Lot {} {}", bx, bz),
                    category: "demo.poi".to_owned(),
                    point: LocalPoint::new(
                        (lot_min.x + lot_max.x) * 0.5,
                        (lot_min.z + lot_max.z) * 0.5,
                    ),
                    source: SourceTag::Procedural,
                });
            }
        }
    }

    tile
}

fn road_class_for_index(index: i32) -> RoadClass {
    if index % 8 == 0 {
        RoadClass::Arterial
    } else if index % 4 == 0 {
        RoadClass::Collector
    } else {
        RoadClass::Local
    }
}

fn tile_seed(id: CityTileId, bx: i32, bz: i32) -> u64 {
    let mut seed = id.x as u64;
    seed ^= (id.z as u64).rotate_left(21);
    seed ^= (bx as u64).rotate_left(37);
    seed ^= (bz as u64).rotate_left(51);
    seed ^ 0x9e37_79b9_7f4a_7c15
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_tile_has_city_features() {
        let tile = generate_demo_tile(
            CityTileId::new(0, 0, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        );

        assert!(!tile.roads.is_empty());
        assert!(!tile.buildings.is_empty());
        assert!(tile.feature_count() >= tile.roads.len() + tile.buildings.len());
    }

    #[test]
    fn demo_tile_is_deterministic() {
        let a = generate_demo_tile(
            CityTileId::new(2, -3, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        );
        let b = generate_demo_tile(
            CityTileId::new(2, -3, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        );

        assert_eq!(a, b);
    }
}
