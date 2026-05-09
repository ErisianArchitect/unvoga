use serde::{Deserialize, Serialize};

const EARTH_RADIUS_M: f64 = 6_378_137.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat_deg: f64,
    pub lon_deg: f64,
}

impl GeoPoint {
    pub const fn new(lat_deg: f64, lon_deg: f64) -> Self {
        Self { lat_deg, lon_deg }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalPoint {
    /// East-west offset in meters from the projector origin.
    pub x: f64,
    /// North-south offset in meters from the projector origin.
    pub z: f64,
}

impl LocalPoint {
    pub const fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalProjector {
    pub origin: GeoPoint,
    origin_lat_rad: f64,
    meters_per_lon_rad: f64,
}

impl LocalProjector {
    /// Local tangent-plane approximation. Keep tiles small enough that
    /// curvature error is irrelevant at gameplay scale.
    pub fn new(origin: GeoPoint) -> Self {
        let origin_lat_rad = origin.lat_deg.to_radians();
        Self {
            origin,
            origin_lat_rad,
            meters_per_lon_rad: EARTH_RADIUS_M * origin_lat_rad.cos(),
        }
    }

    pub fn project(&self, point: GeoPoint) -> LocalPoint {
        let lat_rad = point.lat_deg.to_radians();
        let lon_rad = point.lon_deg.to_radians();
        let origin_lon_rad = self.origin.lon_deg.to_radians();
        LocalPoint {
            x: (lon_rad - origin_lon_rad) * self.meters_per_lon_rad,
            z: (lat_rad - self.origin_lat_rad) * EARTH_RADIUS_M,
        }
    }

    pub fn unproject(&self, point: LocalPoint) -> GeoPoint {
        let lat_rad = point.z / EARTH_RADIUS_M + self.origin_lat_rad;
        let lon_rad = point.x / self.meters_per_lon_rad + self.origin.lon_deg.to_radians();
        GeoPoint {
            lat_deg: lat_rad.to_degrees(),
            lon_deg: lon_rad.to_degrees(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_round_trips_near_origin() {
        let projector = LocalProjector::new(GeoPoint::new(25.7617, -80.1918));
        let point = GeoPoint::new(25.7622, -80.1909);
        let round_trip = projector.unproject(projector.project(point));

        assert!((point.lat_deg - round_trip.lat_deg).abs() < 0.000001);
        assert!((point.lon_deg - round_trip.lon_deg).abs() < 0.000001);
    }
}
