pub mod mesh;
pub mod procedural;
pub mod projection;
pub mod tile;

pub use mesh::{building_mesh, road_mesh};
pub use procedural::{generate_demo_tile, DemoCityConfig};
pub use projection::{GeoPoint, LocalPoint, LocalProjector};
pub use tile::{Building, CityTile, CityTileId, Place, Road, RoadClass, SourceTag};
