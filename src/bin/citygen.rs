use unvoga::core::city::{generate_demo_tile, CityTileId, DemoCityConfig, GeoPoint};

fn main() {
    let origin = GeoPoint::new(25.7617, -80.1918);
    let tile = generate_demo_tile(CityTileId::new(0, 0, 0), origin, DemoCityConfig::default());

    println!(
        "{}",
        serde_json::to_string_pretty(&tile).expect("city tile should serialize")
    );
}
