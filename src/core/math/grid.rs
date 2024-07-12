use crate::core::voxel::coord::Coord;

#[inline]
pub const fn snap(value: i32, snap: i32, offset: i32) -> i32 {
    let snap = snap.abs();
    let snap_offset = offset.rem_euclid(snap);
    let cutoff = (value - snap_offset).rem_euclid(snap);
    value - cutoff
}

pub fn calculate_center_offset(radius: i32, center: Coord) -> Coord {
    let offset = center - Coord::splat(8);
    let radius_offset = (radius - 1) * 16;
    let (x, y, z) = offset.into();
    let (x, y, z) = (
        (x & !0xF) - radius_offset,
        (y & !0xF) - radius_offset,
        (z & !0xF) - radius_offset
    );
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