use rollgrid::rollgrid3d::Bounds3D;

use crate::core::voxel::coord::Coord;

#[inline]
pub const fn snap(value: i32, snap: i32, offset: i32) -> i32 {
    let snap = snap.abs();
    let snap_offset = offset.rem_euclid(snap);
    let cutoff = (value - snap_offset).rem_euclid(snap);
    value - cutoff
}

pub fn calculate_center_offset(chunk_radius: i32, center: Coord, bounds: Option<Bounds3D>) -> Coord {
    let offset = center - Coord::splat(8);
    let radius_offset = (chunk_radius - 1) * 16;
    let (x, y, z) = offset.into();
    let (mut x, mut y, mut z) = (
        (x & !0xF) - radius_offset,
        (y & !0xF) - radius_offset,
        (z & !0xF) - radius_offset
    );
    if let Some(bounds) = bounds {
        let width = (chunk_radius as u32 * 2).min(bounds.width());
        let height = (chunk_radius as u32 * 2).min(bounds.height());
        let depth = (chunk_radius as u32 * 2).min(bounds.depth());
        let xmax = (bounds.max.0 as i64 - width as i64) as i32;
        let ymax = (bounds.max.1 as i64 - width as i64) as i32;
        let zmax = (bounds.max.2 as i64 - width as i64) as i32;
        (x, y, z) = (
            x.min(xmax).max(bounds.min.0),
            y.min(ymax).max(bounds.min.1),
            z.min(zmax).max(bounds.min.2),
        );
    }
    Coord::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use crate::core::math::grid::snap;

    #[test]
    fn snap_test() {
        assert_eq!(snap(9, 10, 5), 5);
        assert_eq!(snap(16, 10, 5), 15);
        assert_eq!(snap(5, 10, 5), 5);
    }
}