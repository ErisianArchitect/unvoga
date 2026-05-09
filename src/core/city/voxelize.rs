use super::projection::LocalPoint;
use super::tile::CityTile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Ground,
    Road,
    Building,
    Poi,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelizeConfig {
    /// Voxels per meter. e.g. `0.25` => 1 voxel = 4 m.
    pub vox_per_meter: f64,
    pub ground_y: i32,
    pub poi_height: i32,
}

impl Default for VoxelizeConfig {
    fn default() -> Self {
        Self {
            vox_per_meter: 0.25,
            ground_y: 0,
            poi_height: 4,
        }
    }
}

pub fn voxelize_tile(
    tile: &CityTile,
    cfg: &VoxelizeConfig,
    mut emit: impl FnMut(i32, i32, i32, BlockKind),
) {
    let s = cfg.vox_per_meter;
    let min = tile.min_local();
    let max = tile.max_local();
    let (x0, x1) = (m_to_v(min.x, s), m_to_v(max.x, s));
    let (z0, z1) = (m_to_v(min.z, s), m_to_v(max.z, s));

    // Ground plane.
    for z in z0..x_lt(z0, z1) {
        for x in x0..x_lt(x0, x1) {
            emit(x, cfg.ground_y, z, BlockKind::Ground);
        }
    }

    // Roads: thick polylines, one voxel above ground.
    let road_y = cfg.ground_y + 1;
    for road in &tile.roads {
        let half = (road.width_m as f64 * 0.5).max(0.5);
        let half_v = (half * s).max(0.5);
        for win in road.centerline.windows(2) {
            rasterize_thick_segment(win[0], win[1], half_v, s, |x, z| {
                if x >= x0 && x < x1 && z >= z0 && z < z1 {
                    emit(x, road_y, z, BlockKind::Road);
                }
            });
        }
    }

    // Buildings: extruded footprint columns.
    let base_y = cfg.ground_y + 1;
    for b in &tile.buildings {
        if b.footprint.len() < 3 {
            continue;
        }
        let h_v = ((b.height_m as f64 * s).round() as i32).max(1);
        let (bx0, bx1, bz0, bz1) = poly_bbox(&b.footprint, s);
        for z in bx0_clamp(bz0, z0)..bx1_clamp(bz1, z1) {
            for x in bx0_clamp(bx0, x0)..bx1_clamp(bx1, x1) {
                let cx = (x as f64 + 0.5) / s;
                let cz = (z as f64 + 0.5) / s;
                if point_in_polygon(cx, cz, &b.footprint) {
                    for y in base_y..base_y + h_v {
                        emit(x, y, z, BlockKind::Building);
                    }
                }
            }
        }
    }

    // Places: pillars.
    let poi_base = cfg.ground_y + 1;
    for p in &tile.places {
        let x = m_to_v(p.point.x, s);
        let z = m_to_v(p.point.z, s);
        if x < x0 || x >= x1 || z < z0 || z >= z1 {
            continue;
        }
        for y in poi_base..poi_base + cfg.poi_height {
            emit(x, y, z, BlockKind::Poi);
        }
    }
}

fn m_to_v(m: f64, s: f64) -> i32 {
    (m * s).floor() as i32
}

#[inline]
fn x_lt(_lo: i32, hi: i32) -> i32 {
    hi
}

#[inline]
fn bx0_clamp(v: i32, lo: i32) -> i32 {
    v.max(lo)
}

#[inline]
fn bx1_clamp(v: i32, hi: i32) -> i32 {
    v.min(hi)
}

fn poly_bbox(points: &[LocalPoint], s: f64) -> (i32, i32, i32, i32) {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for p in points {
        x_min = x_min.min(p.x);
        x_max = x_max.max(p.x);
        z_min = z_min.min(p.z);
        z_max = z_max.max(p.z);
    }
    (
        m_to_v(x_min, s),
        m_to_v(x_max, s) + 1,
        m_to_v(z_min, s),
        m_to_v(z_max, s) + 1,
    )
}

/// Standard ray-cast point-in-polygon (xz plane).
fn point_in_polygon(px: f64, pz: f64, poly: &[LocalPoint]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, zi) = (poly[i].x, poly[i].z);
        let (xj, zj) = (poly[j].x, poly[j].z);
        if (zi > pz) != (zj > pz) {
            let xc = (xj - xi) * (pz - zi) / (zj - zi) + xi;
            if px < xc {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Rasterize a thick line segment in voxel space; emit voxel (x,z) cells whose
/// center is within `half_v` voxels of the segment.
fn rasterize_thick_segment(
    a: LocalPoint,
    b: LocalPoint,
    half_v: f64,
    s: f64,
    mut emit: impl FnMut(i32, i32),
) {
    let ax = a.x * s;
    let az = a.z * s;
    let bx = b.x * s;
    let bz = b.z * s;
    let pad = half_v.ceil() as i32 + 1;
    let xlo = ax.min(bx).floor() as i32 - pad;
    let xhi = ax.max(bx).ceil() as i32 + pad;
    let zlo = az.min(bz).floor() as i32 - pad;
    let zhi = az.max(bz).ceil() as i32 + pad;
    let dx = bx - ax;
    let dz = bz - az;
    let len2 = dx * dx + dz * dz;
    let half2 = half_v * half_v;
    for z in zlo..=zhi {
        for x in xlo..=xhi {
            let cx = x as f64 + 0.5;
            let cz = z as f64 + 0.5;
            let t = if len2 > 0.0 {
                (((cx - ax) * dx + (cz - az) * dz) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let qx = ax + t * dx;
            let qz = az + t * dz;
            let dxq = cx - qx;
            let dzq = cz - qz;
            if dxq * dxq + dzq * dzq <= half2 {
                emit(x, z);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::city::procedural::{generate_demo_tile, DemoCityConfig};
    use crate::core::city::projection::GeoPoint;
    use crate::core::city::tile::CityTileId;

    fn tally(tile: &CityTile, cfg: &VoxelizeConfig) -> std::collections::HashMap<BlockKind, usize> {
        let mut counts: std::collections::HashMap<BlockKind, usize> = Default::default();
        voxelize_tile(tile, cfg, |_, _, _, k| *counts.entry(k).or_insert(0) += 1);
        counts
    }

    #[test]
    fn demo_tile_voxelizes_to_all_kinds() {
        let tile = generate_demo_tile(
            CityTileId::new(0, 0, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        );
        let counts = tally(&tile, &VoxelizeConfig::default());
        assert!(counts[&BlockKind::Ground] > 0);
        assert!(counts[&BlockKind::Road] > 0);
        assert!(counts[&BlockKind::Building] > 0);
        // Demo POIs are sparse but present in the default tile.
        assert!(counts.get(&BlockKind::Poi).copied().unwrap_or(0) > 0);
    }

    #[test]
    fn ground_plane_covers_tile() {
        let tile = generate_demo_tile(
            CityTileId::new(0, 0, 0),
            GeoPoint::new(0.0, 0.0),
            DemoCityConfig::default(),
        );
        let cfg = VoxelizeConfig::default();
        let side_m = tile.tile_size_m;
        let expected = (side_m * cfg.vox_per_meter).floor() as usize;
        let counts = tally(&tile, &cfg);
        assert_eq!(counts[&BlockKind::Ground], expected * expected);
    }

    #[test]
    fn point_in_polygon_axis_aligned_square() {
        let sq = vec![
            LocalPoint::new(0.0, 0.0),
            LocalPoint::new(10.0, 0.0),
            LocalPoint::new(10.0, 10.0),
            LocalPoint::new(0.0, 10.0),
        ];
        assert!(point_in_polygon(5.0, 5.0, &sq));
        assert!(!point_in_polygon(-1.0, 5.0, &sq));
        assert!(!point_in_polygon(11.0, 5.0, &sq));
    }
}
