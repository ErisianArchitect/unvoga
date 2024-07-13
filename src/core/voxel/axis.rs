use bytemuck::{NoUninit, Pod, Zeroable};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NoUninit)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2
}

#[test]
fn d() {
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let axes2: &[u8] = bytemuck::cast_slice(&axes);
}