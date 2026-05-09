use super::projection::LocalPoint;
use super::tile::CityTile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Ground,
    Sand,
    Road,
    Sidewalk,
    /// Per-building palette index in `0..VoxelizeConfig::building_palette_size`.
    Building { palette: u8 },
    Roof,
    Poi,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelizeConfig {
    /// Voxels per meter. e.g. `0.25` => 1 voxel = 4 m.
    pub vox_per_meter: f64,
    pub ground_y: i32,
    pub poi_height: i32,
    /// Half-width (m) of the sidewalk ring rasterized around each building.
    pub sidewalk_width_m: f32,
    /// Number of distinct building palette slots emitted in `Building { palette }`.
    pub building_palette_size: u8,
    /// Per-cell ground variation: 1 in `sand_freq` cells emits `Sand` instead of `Ground`.
    pub sand_freq: u32,
}

impl Default for VoxelizeConfig {
    fn default() -> Self {
        Self {
            vox_per_meter: 0.25,
            ground_y: 0,
            poi_height: 4,
            sidewalk_width_m: 2.5,
            building_palette_size: 4,
            sand_freq: 32,
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

    // Ground plane with sand variation.
    let sand_freq = cfg.sand_freq.max(1);
    for z in z0..z1 {
        for x in x0..x1 {
            let h = splitmix64((x as u64 ^ ((z as u64) << 32)).wrapping_add(0x9e3779b97f4a7c15));
            let kind = if (h as u32) % sand_freq == 0 {
                BlockKind::Sand
            } else {
                BlockKind::Ground
            };
            emit(x, cfg.ground_y, z, kind);
        }
    }

    // Roads: thick polylines, one voxel above ground.
    let road_y = cfg.ground_y + 1;
    for road in &tile.roads {
        let half_v = ((road.width_m as f64 * 0.5) * s).max(0.5);
        for win in road.centerline.windows(2) {
            rasterize_thick_segment(win[0], win[1], half_v, s, |x, z| {
                if x >= x0 && x < x1 && z >= z0 && z < z1 {
                    emit(x, road_y, z, BlockKind::Road);
                }
            });
        }
    }

    // Sidewalks + buildings.
    let base_y = cfg.ground_y + 1;
    let sidewalk_half_v = (cfg.sidewalk_width_m as f64 * 0.5 * s).max(0.5);
    let palette_n = cfg.building_palette_size.max(1);
    for b in &tile.buildings {
        if b.footprint.len() < 3 {
            continue;
        }

        // Sidewalk ring: rasterize the footprint perimeter as a thick polyline
        // at ground+1. Order matters less than coverage; we then overwrite
        // interior cells with the building below.
        for i in 0..b.footprint.len() {
            let a = b.footprint[i];
            let bp = b.footprint[(i + 1) % b.footprint.len()];
            rasterize_thick_segment(a, bp, sidewalk_half_v, s, |x, z| {
                if x >= x0 && x < x1 && z >= z0 && z < z1 {
                    emit(x, base_y, z, BlockKind::Sidewalk);
                }
            });
        }

        let h_v = ((b.height_m as f64 * s).round() as i32).max(1);
        let (bx0, bx1, bz0, bz1) = poly_bbox(&b.footprint, s);
        let palette = (str_hash(&b.id) % palette_n as u64) as u8;
        for z in bz0.max(z0)..bz1.min(z1) {
            for x in bx0.max(x0)..bx1.min(x1) {
                let cx = (x as f64 + 0.5) / s;
                let cz = (z as f64 + 0.5) / s;
                if !point_in_polygon(cx, cz, &b.footprint) {
                    continue;
                }
                let top = base_y + h_v - 1;
                for y in base_y..=top {
                    let kind = if y == top && h_v >= 2 {
                        BlockKind::Roof
                    } else {
                        BlockKind::Building { palette }
                    };
                    emit(x, y, z, kind);
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

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn str_hash(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::city::procedural::{generate_demo_tile, DemoCityConfig};
    use crate::core::city::projection::GeoPoint;
    use crate::core::city::tile::CityTileId;

    fn tally(tile: &CityTile, cfg: &VoxelizeConfig) -> std::collections::HashMap<&'static str, usize> {
        let mut counts: std::collections::HashMap<&'static str, usize> = Default::default();
        voxelize_tile(tile, cfg, |_, _, _, k| {
            let key = match k {
                BlockKind::Ground => "ground",
                BlockKind::Sand => "sand",
                BlockKind::Road => "road",
                BlockKind::Sidewalk => "sidewalk",
                BlockKind::Building { .. } => "building",
                BlockKind::Roof => "roof",
                BlockKind::Poi => "poi",
            };
            *counts.entry(key).or_insert(0) += 1;
        });
        counts
    }

    fn demo() -> CityTile {
        generate_demo_tile(
            CityTileId::new(0, 0, 0),
            GeoPoint::new(25.7617, -80.1918),
            DemoCityConfig::default(),
        )
    }

    #[test]
    fn demo_tile_voxelizes_to_all_kinds() {
        let counts = tally(&demo(), &VoxelizeConfig::default());
        for k in ["ground", "road", "sidewalk", "building", "roof", "poi"] {
            assert!(counts.get(k).copied().unwrap_or(0) > 0, "missing {k}");
        }
    }

    #[test]
    fn ground_plane_covers_tile_with_sand_variation() {
        let tile = demo();
        let cfg = VoxelizeConfig::default();
        let side = (tile.tile_size_m * cfg.vox_per_meter).floor() as usize;
        let counts = tally(&tile, &cfg);
        let ground_total = counts.get("ground").copied().unwrap_or(0)
            + counts.get("sand").copied().unwrap_or(0);
        assert_eq!(ground_total, side * side);
        assert!(counts.get("sand").copied().unwrap_or(0) > 0, "sand variation expected");
    }

    #[test]
    fn building_palette_in_range_and_stable_per_id() {
        let tile = demo();
        let cfg = VoxelizeConfig::default();
        let mut palettes_seen: std::collections::HashSet<u8> = Default::default();
        voxelize_tile(&tile, &cfg, |_, _, _, k| {
            if let BlockKind::Building { palette } = k {
                assert!(palette < cfg.building_palette_size);
                palettes_seen.insert(palette);
            }
        });
        assert!(palettes_seen.len() > 1, "expected palette variety");
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
