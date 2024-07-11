#[inline]
pub const fn snap(value: i32, snap: i32, offset: i32) -> i32 {
    let snap = snap.abs();
    let snap_offset = offset.rem_euclid(snap);
    let cutoff = (value - snap_offset).rem_euclid(snap);
    value - cutoff
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