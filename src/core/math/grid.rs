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
        let width = (chunk_radius as u32 * 32).min(bounds.width());
        let height = (chunk_radius as u32 * 32).min(bounds.height());
        let depth = (chunk_radius as u32 * 32).min(bounds.depth());
        let xmax = (bounds.max.0 as i64 - width as i64) as i32;
        let ymax = (bounds.max.1 as i64 - height as i64) as i32;
        // println!("ymax: {ymax} {y} {height}");
        let zmax = (bounds.max.2 as i64 - depth as i64) as i32;
        (x, y, z) = (
            x.min(xmax).max(bounds.min.0),
            y.min(ymax).max(bounds.min.1),
            z.min(zmax).max(bounds.min.2),
        );
    }
    Coord::new(x, y, z)
}

#[inline(always)]
pub fn round_up_to_multiple_of_32(value: i32) -> i32 {
    (value + 31) & -32
}

#[inline(always)]
pub fn round_down_to_multiple_of_32(value: i32) -> i32 {
    value & -32
}

/// chunk_width is the width in chunks, that is, the number of chunks wide.
#[inline(always)]
pub fn calculate_region_requirement(chunk_width: i32) -> i32 {
    let rd32 = round_up_to_multiple_of_32(chunk_width);
    let rdw = rd32 / 32;
    (rdw + 1)
}

/// world_chunk_min is the chunk coordinate of the world minimum bound.
#[inline(always)]
pub fn calculate_region_min(world_chunk_min: (i32, i32)) -> (i32, i32) {
    (
        round_down_to_multiple_of_32(world_chunk_min.0) / 32,
        round_down_to_multiple_of_32(world_chunk_min.1) / 32
    )
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