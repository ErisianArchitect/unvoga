/// A simple storage container for data associated with the 6 sides of a cube.
pub struct Faces<T> {
    /// (-1, 0, 0)
    pub neg_x: T,
    /// (0, -1, 0)
    pub neg_y: T,
    /// (0, 0, -1)
    pub neg_z: T,
    /// (1, 0, 0)
    pub pos_x: T,
    /// (0, 1, 0)
    pub pos_y: T,
    /// (0, 0, 1)
    pub pos_z: T
}

impl<T: Clone> Clone for Faces<T> {
    fn clone(&self) -> Self {
        Self {
            neg_x: self.neg_x.clone(),
            neg_y: self.neg_y.clone(),
            neg_z: self.neg_z.clone(),
            pos_x: self.pos_x.clone(),
            pos_y: self.pos_y.clone(),
            pos_z: self.pos_z.clone()
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.neg_x = source.neg_x.clone();
        self.neg_y = source.neg_y.clone();
        self.neg_z = source.neg_z.clone();
        self.pos_x = source.pos_x.clone();
        self.pos_y = source.pos_y.clone();
        self.pos_z = source.pos_z.clone();
    }
}

impl<T: Copy> Copy for Faces<T> {}