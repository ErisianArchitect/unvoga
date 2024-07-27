#![allow(unused)]

pub fn index2<const W: i32>(x: i32, y: i32) -> usize {
    let x = x.rem_euclid(W);
    let y = y.rem_euclid(W);
    (y * W + x) as usize
}


pub fn index3<const W: i32>(x: i32, y: i32, z: i32) -> usize {
    let x = x.rem_euclid(W);
    let y = y.rem_euclid(W);
    let z = z.rem_euclid(W);
    (y * W*W + z * W + x) as usize
}

/// Returns (min, max)
pub fn minmax<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a <= b { (a, b) } else { (b, a) }
}