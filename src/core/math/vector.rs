use bevy::math::Vec3;



/// Calculate the normal of a triangle.
pub fn calculate_tri_normal(tri: &[Vec3]) -> Vec3 {
    assert_eq!(tri.len(), 3);
    let a = tri[1] - tri[0];
    let b = tri[2] - tri[0];
    let nx = a.y * b.z - a.z * b.y;
    let ny = a.z * b.x - a.x * b.z;
    let nz = a.x * b.y - a.y * b.x;
    Vec3::new(nx, ny, nz).normalize()
}

#[cfg(test)]
mod tests {
    use bevy::math::vec3;

    use super::*;
    #[test]
    fn norm_test() {
        let tri = [
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 0.0, 0.0),
        ];
        let norm = calculate_tri_normal(&tri);
        println!("{norm}");
    }
}