#![allow(unused)]

// Most of this code in this file wasn't written because I explicitly
// needed it, I mostly wrote it out of boredom. Great for reuse, though.
pub struct Grid<const W: usize, const H: usize, T> {
    cells: Vec<T>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Grid8x8Bitmask {
    bitmask: [u8; 8]
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Grid16x16Bitmask {
    bitmask: [u16; 16]
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Grid32x32Bitmask {
    bitmask: Box<[u32; 32]>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid64x64Bitmask {
    bitmask: Box<[u64; 64]>
}

/// A Grid type that is useful for implementations of infinite worlds.
pub struct RollingGrid<const W: usize, const H: usize, T> {
    cells: Vec<Option<T>>,
    roll: (usize, usize),
    offset: (i64, i64),
}

/// When using itertools::iproduct, you sometimes want to reverse the order
/// of values. Sorry, I'm not wording this well.
/// `itertools::iproduct!(0..2, 2..4)` will produce:
/// (0,2)
/// (0,3)
/// (1,2)
/// (1,3)
/// Notice how the 2..4 is iterated first, then 0..2.
/// Sometimes you want the 0..2 to come first, so what do you do?
/// You reverse to order of the ranges and also reverse the order of
/// your parameters.
/// `itertools::iproduct!(2..4, 0..2).for_each(|(y, x)| ())`
/// But reversing the order of your parameters makes it harder to work
/// with closures that accept (x, y) ordering.
/// So this function will swap the order of x and y.
/// `itertools::iproduct!(2..4, 0..2).map(iprod_swap).for_each(|(x, y)| ())`
/// It would have been easier to read the code to see what it does, but
/// then you wouldn't have known what it was for.
#[inline(always)]
fn iproduct_rev<T>(input: (T, T)) -> (T, T) {
    (input.1, input.0)
}

impl<const W: usize, const H: usize, T> Grid<W,H,T> {
    /// Create a new [Grid] with an initialize function.
    /// ```rust,no_run
    /// fn init(position: (i64, i64)) -> Option<T>
    /// ```
    /// Where the return value of init is the value you would like to store.
    pub fn new_with_init<F: FnMut((usize, usize)) -> T>(init: F) -> Self {
        let mut init = init;
        Self {
            cells: itertools::iproduct!(0..H, 0..W)
                .map(iproduct_rev)
                .map(init).collect()
        }
    }
}

impl<const W: usize, const H: usize, T: Default> Default for Grid<W,H,T> {
    fn default() -> Self {
        Self {
            cells: (0..W*H).map(|_| T::default()).collect()
        }
    }
}

impl<const W: usize, const H: usize, T: Default> Grid<W,H,T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const W: usize, const H: usize, T: Copy> Grid<W,H,T> {
    pub fn get_copy(&self, x: usize, y: usize) -> Option<T> {
        let index = self.index(x, y)?;
        Some(self.cells[index])
    }
}

impl<const W: usize, const H: usize, T> Grid<W,H,T> {
    pub fn len(&self) -> usize {
        W*H
    }

    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= W || y >= H {
            return None;
        }
        Some((y * W) + x)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        let index = self.index(x, y)?;
        Some(&self.cells[index])
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let index = self.index(x, y)?;
        Some(&mut self.cells[index])
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) -> Option<T>  {
        let index = self.index(x, y)?;
        let mut old = value;
        std::mem::swap(&mut old, &mut self.cells[index]);
        Some(old)
    }

    pub fn iter<'a>(&'a self) -> GridIterator<'a,W,H,T> {
        GridIterator {
            grid: self,
            offset: (0, 0),
            index: 0,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> GridMutIterator<'a,W,H,T> {
        GridMutIterator {
            grid: self,
            offset: (0, 0),
            index: 0,
        }
    }
}

pub struct GridIterator<'a, const W: usize, const H: usize, T> {
    grid: &'a Grid<W,H,T>,
    offset: (usize, usize),
    index: usize,
}

impl<'a, const W: usize, const H: usize, T> Iterator for GridIterator<'a,W,H,T> {
    type Item = ((usize, usize), &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.grid.len() - self.index;
        (remaining, Some(remaining))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let (mut cx, mut cy) = self.offset;
        if cx >= W
        || cy >= H {
            return None;
        }

        let result = ((cx, cy), self.grid.get(cx, cy)?);
        self.index += 1;
        cx += 1;
        if cx >= W {
            cx = 0;
            cy += 1;
        }
        self.offset = (cx, cy);
        Some(result)
    }
}

pub struct GridMutIterator<'a, const W: usize, const H: usize, T> {
    grid: &'a mut Grid<W,H,T>,
    offset: (usize, usize),
    index: usize,
}

impl<'a, const W: usize, const H: usize, T> Iterator for GridMutIterator<'a,W,H,T> {
    type Item = ((usize, usize), &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.grid.len() - self.index;
        (remaining, Some(remaining))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let (mut cx, mut cy) = self.offset;
        let pos = (cx, cy);
        let index = self.grid.index(cx, cy)?;
        self.index += 1;
        cx += 1;
        if cx >= W {
            cx = 0;
            cy += 1;
        }
        self.offset = (cx, cy);
        unsafe {
            let cells_ptr = self.grid.cells.as_mut_ptr();
            Some((pos, &mut *cells_ptr.add(index)))
        }
    }
}

impl<const W: usize, const H: usize, T> std::ops::Index<(usize, usize)> for Grid<W,H,T> {
    type Output = T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        self.get(x, y).expect("Out of bounds.")
    }
}

impl<const W: usize, const H: usize, T> std::ops::IndexMut<(usize, usize)> for Grid<W,H,T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        self.get_mut(x, y).expect("Out of bounds.")
    }
}

impl<const W: usize, const H: usize, T: Clone> Clone for Grid<W,H,T> {
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone()
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.cells.iter_mut().enumerate().for_each(|(index, value)| {
            *value = source.cells[index].clone();
        });
    }
}

impl Grid8x8Bitmask {
    pub fn new() -> Self {
        Self::default()
    }
        
    pub fn new_with_init<F: FnMut((usize, usize)) -> bool>(init: F) -> Self {
        let mut init = init;
        let mut masks: [u8; 8] = [0; 8];
        itertools::iproduct!(0..8, 0..8)
            .map(iproduct_rev)
            .for_each(|(x, y)| {
                if init((x, y)) {
                    masks[y] = masks[y] | (1 << x);
                }
            });
        Self {
            bitmask: masks
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= 8 || y >= 8 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        (sub & (1 << x)) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        if x >= 8 || y >= 8 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        let old = (sub & (1 << x)) != 0;
        self.bitmask[y] = if value {
            sub | (1 << x)
        } else {
            sub & !(1 << x)
        };
        old
    }

    pub fn to_grid(&self) -> Grid<8, 8, bool> {
        Grid::<8, 8, bool>::new_with_init(|(x, y)| {
            self.get(x, y)
        })
    }
}

impl Grid16x16Bitmask {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn new_with_init<F: FnMut((usize, usize)) -> bool>(init: F) -> Self {
        let mut init = init;
        let mut masks: [u16; 16] = [0; 16];
        itertools::iproduct!(0..16, 0..16)
            .map(iproduct_rev)
            .for_each(|(x, y)| {
                if init((x, y)) {
                    masks[y] = masks[y] | (1 << x);
                }
            });
        Self {
            bitmask: masks
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= 16 || y >= 16 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        (sub & (1 << x)) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        if x >= 16 || y >= 16 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        let old = (sub & (1 << x)) != 0;
        self.bitmask[y] = if value {
            sub | (1 << x)
        } else {
            sub & !(1 << x)
        };
        old
    }

    
    pub fn to_grid(&self) -> Grid<16, 16, bool> {
        Grid::<16, 16, bool>::new_with_init(|(x, y)| {
            self.get(x, y)
        })
    }
}

impl Grid32x32Bitmask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_init<F: FnMut((usize, usize)) -> bool>(init: F) -> Self {
        let mut init = init;
        let mut masks: Box<[u32; 32]> = Box::new([0; 32]);
        itertools::iproduct!(0..32, 0..32)
            .map(iproduct_rev)
            .for_each(|(x, y)| {
                if init((x, y)) {
                    masks[y] = masks[y] | (1 << x);
                }
            });
        Self {
            bitmask: masks
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= 32 || y >= 32 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        (sub & (1 << x)) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        if x >= 32 || y >= 32 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        let old = (sub & (1 << x)) != 0;
        self.bitmask[y] = if value {
            sub | (1 << x)
        } else {
            sub & !(1 << x)
        };
        old
    }

        
    pub fn to_grid(&self) -> Grid<32, 32, bool> {
        Grid::<32, 32, bool>::new_with_init(|(x, y)| {
            self.get(x, y)
        })
    }
}

impl Default for Grid64x64Bitmask {
    fn default() -> Self {
        Self {
            bitmask: Box::new([0; 64])
        }
    }
}

impl Grid64x64Bitmask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_init<F: FnMut((usize, usize)) -> bool>(init: F) -> Self {
        let mut init = init;
        let mut masks: Box<[u64; 64]> = Box::new([0; 64]);
        itertools::iproduct!(0..64, 0..64)
            .map(iproduct_rev)
            .for_each(|(x, y)| {
                if init((x, y)) {
                    masks[y] = masks[y] | (1 << x);
                }
            });
        Self {
            bitmask: masks
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= 64 || y >= 64 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        (sub & (1 << x)) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        if x >= 64 || y >= 64 {
            panic!("Out of bounds.");
        }
        let sub = self.bitmask[y];
        let old = (sub & (1 << x)) != 0;
        self.bitmask[y] = if value {
            sub | (1 << x)
        } else {
            sub & !(1 << x)
        };
        old
    }
    
    pub fn to_grid(&self) -> Grid<64, 64, bool> {
        Grid::<64, 64, bool>::new_with_init(|(x, y)| {
            self.get(x, y)
        })
    }
}

impl From<Grid<8, 8, bool>> for Grid8x8Bitmask {
    fn from(value: Grid<8, 8, bool>) -> Self {
        Self::new_with_init(|(x, y)| *value.get(x, y).unwrap())
    }
}

impl From<Grid<16, 16, bool>> for Grid16x16Bitmask {
    fn from(value: Grid<16, 16, bool>) -> Self {
        Self::new_with_init(|(x, y)| *value.get(x, y).unwrap())
    }
}

impl From<Grid<32, 32, bool>> for Grid32x32Bitmask {
    fn from(value: Grid<32, 32, bool>) -> Self {
        Self::new_with_init(|(x, y)| *value.get(x, y).unwrap())
    }
}

impl From<Grid<64, 64, bool>> for Grid64x64Bitmask {
    fn from(value: Grid<64, 64, bool>) -> Self {
        Self::new_with_init(|(x, y)| *value.get(x, y).unwrap())
    }
}

impl From<Grid8x8Bitmask> for Grid<8, 8, bool> {
    fn from(value: Grid8x8Bitmask) -> Self {
        Self::new_with_init(|(x, y)| value.get(x, y))
    }
}

impl From<Grid16x16Bitmask> for Grid<16, 16, bool> {
    fn from(value: Grid16x16Bitmask) -> Self {
        Self::new_with_init(|(x, y)| value.get(x, y))
    }
}

impl From<Grid32x32Bitmask> for Grid<32, 32, bool> {
    fn from(value: Grid32x32Bitmask) -> Self {
        Self::new_with_init(|(x, y)| value.get(x, y))
    }
}

impl From<Grid64x64Bitmask> for Grid<64, 64, bool> {
    fn from(value: Grid64x64Bitmask) -> Self {
        Self::new_with_init(|(x, y)| value.get(x, y))
    }
}

impl Grid<8, 8, bool> {
    pub fn to_bitmask(&self) -> Grid8x8Bitmask {
        Grid8x8Bitmask::new_with_init(|(x, y)| {
            *self.get(x, y).unwrap()
        })
    }
}

impl Grid<16, 16, bool> {
    pub fn to_bitmask(&self) -> Grid16x16Bitmask {
        Grid16x16Bitmask::new_with_init(|(x, y)| {
            *self.get(x, y).unwrap()
        })
    }
}

impl Grid<32, 32, bool> {
    pub fn to_bitmask(&self) -> Grid32x32Bitmask {
        Grid32x32Bitmask::new_with_init(|(x, y)| {
            *self.get(x, y).unwrap()
        })
    }
}

impl Grid<64, 64, bool> {
    pub fn to_bitmask(&self) -> Grid64x64Bitmask {
        Grid64x64Bitmask::new_with_init(|(x, y)| {
            *self.get(x, y).unwrap()
        })
    }
}

impl<const W: usize, const H: usize, T: Default> RollingGrid<W, H, T> {
    pub fn new_with_default(offset: (i64, i64)) -> Self {
        Self::new_with_init(offset, |_| Some(Default::default()))
    }
}

impl<const W: usize, const H: usize, T: Copy> RollingGrid<W,H,T> {
    pub fn new_with_copy(offset: (i64, i64), init: T) -> Self {
        Self::new_with_init(offset, |_| Some(init))
    }
}

impl<const W: usize, const H: usize, T: Clone> RollingGrid<W,H,T> {
    pub fn new_with_clone(offset: (i64, i64), init: &T) -> Self {
        Self::new_with_init(offset, |_| Some(init.clone()))
    }
}

impl<const W: usize, const H: usize, T> RollingGrid<W,H,T> {

    /// This will not automatically intialize instances stored within.
    /// You must manually set them using `self.set()`.
    pub fn new(offset: (i64, i64)) -> Self {
        let mul = W.checked_mul(H).expect("Size is too large.");
        if mul == 0 {
            panic!("Width/Height cannot be 0.");
        }
        if mul > i64::MAX as usize {
            panic!("Width or height is too large.");
        }
        Self {
            cells: (0..W*H).map(|_| None).collect(),
            offset,
            roll: (0, 0),
        }
    }

    /// Create a new [RollingGrid] with an initialize function.
    /// ```rust,no_run
    /// fn init(position: (i64, i64)) -> Option<T>
    /// ```
    /// Where the return value of init is the value you would like to store.
    pub fn new_with_init<F: FnMut((i64, i64)) -> Option<T>>(offset: (i64, i64), init: F) -> Self {
        let mut init = init;
        let mul = W.checked_mul(H).expect("Size is too large.");
        if mul == 0 {
            panic!("Width/Height cannot be 0.");
        }
        if mul > i64::MAX as usize {
            panic!("Width or height is too large.");
        }
        if offset.0.checked_add(W as i64).is_none()
        || offset.1.checked_add(H as i64).is_none() {
            panic!("Offset too close to the maximum bound.");
        }
        Self {
            offset,
            roll: (0, 0),
            cells: itertools::iproduct!(0..H as i64, 0..W as i64)
                .map(|(y, x)| (x + offset.0, y + offset.1))
                .map(|pos| {
                    init(pos)
                }).collect()
        }
    }

    /// The offset of the rolling grid.
    pub fn offset(&self) -> (i64, i64) {
        self.offset
    }

    /// The width of the rolling grid.
    pub fn width(&self) -> usize {
        W
    }

    /// The height of the rolling grid.
    pub fn height(&self) -> usize {
        H
    }

    /// The number of elements that this rolling grid stores.
    pub fn len(&self) -> usize {
        W * H
    }

    /// Get the index at the offset relative to the current offset.
    /// That means that if the [RollingGrid]'s current offset is (3, 4)
    /// and you request the element at (3, 4), you will get the index 0.
    pub fn offset_index(&self, (x, y): (i64, i64)) -> Option<usize> {
        let (mx, my) = self.offset;
        let width = W as i64;
        let height = H as i64;
        if x >= mx + width
        || y >= my + height
        || x < mx
        || y < my {
            return  None;
        }
        // Adjust x and y
        let nx = x - mx;
        let ny = y - my;
        // Roll x and y
        let (roll_x, roll_y) = (self.roll.0 as i64, self.roll.1 as i64);
        let rx = (nx + roll_x).rem_euclid(width);
        let ry = (ny + roll_y).rem_euclid(height);
        Some((W * ry as usize) + rx as usize)
    }

    /// Get an immutable reference to the value in the slot if it exists.
    pub fn get(&self, (x, y): (i64, i64)) -> Option<&T> {
        let index = self.offset_index((x, y))?;
        if let Some(cell) = &self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    /// Get a mutable reference to the value in the slot if it exists.
    pub fn get_mut(&mut self, (x, y): (i64, i64)) -> Option<&mut T> {
        let index = self.offset_index((x, y))?;
        if let Some(cell) = &mut self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    /// Internal function for getting the option value of the slot.
    fn get_opt_mut(&mut self, (x, y): (i64, i64)) -> Option<&mut Option<T>> {
        let index = self.offset_index((x, y))?;
        Some(&mut self.cells[index])
    }

    /// Set the value and return the old.
    pub fn set(&mut self, (x, y): (i64, i64), value: T) -> Option<T> {
        let cell = self.get_mut((x, y))?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }

    /// Internal function for setting the option value of the slot.
    fn set_opt(&mut self, (x, y): (i64, i64), value: Option<T>) -> Option<Option<T>> {
        let cell = self.get_opt_mut((x, y))?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }

    /// Translate the grid by offset amount with a reload function.
    /// Signature of the reload function is as follows:
    /// ```rust,no_run
    /// fn reload(old_position: (i64, i64), new_position: (i64, i64), old_value: T) -> Option<T>
    /// ```
    /// Where the return value of `reload` is the new value for that slot.
    pub fn translate<F: FnMut((i64, i64), (i64, i64), Option<T>) -> Option<T>>(&mut self, offset: (i64, i64), reload: F) {
        let (curx, cury) = self.offset;
        let (ox, oy) = offset;
        let position = (
            curx + ox,
            cury + oy
        );
        self.reposition(position, reload);
    }

    /// Reposition the offset of the grid and reload the slots that are changed.
    /// Signature of the reload function is as follows:
    /// ```rust,no_run
    /// fn reload(old_position: (i64, i64), new_position: (i64, i64), old_value: T) -> Option<T>
    /// ```
    /// Where the return value of `reload` is the new value for that slot.
    pub fn reposition<F: FnMut((i64, i64), (i64, i64), Option<T>) -> Option<T>>(&mut self, position: (i64, i64), reload: F) {
        // #[inline(always)]
        // fn offset_value(value: i64, offset: i64, wrap: i64) -> i64 {
        //     (value + offset).rem_euclid(wrap)
        // }
        let (curx, cury) = self.offset;
        let (px, py) = position;
        let offset = (
            px - curx,
            py - cury
        );
        if offset == (0, 0) {
            return;
        }
        let mut reload = reload;
        let width = W as i64;
        let height = H as i64;
        let (offset_x, offset_y) = offset;
        let (old_x, old_y) = self.offset;
        let (new_x, new_y) = (old_x + offset_x, old_y + offset_y);
        self.offset = (new_x, new_y);
        // Offset is within bounds, so that means that the grid will be rolled.
        // This allows for bounded reloading of the grid elements.
        // If rolling causes a section to remain on the grid, that section will not be reloaded.
        // Only the elements that are considered new will be reloaded.
        if offset_x.abs() < width && offset_y.abs() < height {
            let (roll_x, roll_y) = (
                self.roll.0 as i64,
                self.roll.1 as i64
            );
            let (wrapped_offset_x, wrapped_offset_y) = (
                offset_x.rem_euclid(width),
                offset_y.rem_euclid(height)
            );
            // Update the roll so that we reduce reloading.
            // Without using the roll functionality, this function would demand to reload
            // every single cell, even if it only needed to reload 8 out of 64 cells.
            let new_rolled_x = (roll_x + wrapped_offset_x).rem_euclid(width);
            let new_rolled_y = (roll_y + wrapped_offset_y).rem_euclid(height);
            self.roll = (new_rolled_x as usize, new_rolled_y as usize);
            let right = new_x + width;
            let bottom = new_y + height;
            // Calculate ranges
            // Combining new_x_range and new_y_range gets the corner.
            // The partition on either the left or right side
            let new_x_range = if offset_x >= 0 {
                (right - offset_x)..right
            } else {
                new_x..new_x-offset_x
            };
            let new_x_range_y_range = if offset_y >= 0 {
                new_y..(bottom - offset_y)
            } else {
                new_y-offset_y..bottom
            };
            // The partition on either the top or the bottom.
            let new_y_range = if offset_y >= 0 {
                (bottom - offset_y)..bottom
            } else {
                new_y..new_y-offset_y
            };
            let new_y_range_x_range = if offset_x >= 0 {
                new_x..(right - offset_x)
            } else {
                new_x-offset_x..right
            };
            // The left/right partition
            for (yi, y) in new_x_range_y_range.clone().enumerate() {
                for (xi, x) in new_x_range.clone().enumerate() {
                    let prior_x = if offset_x >= 0 {
                        old_x + xi as i64
                    } else {
                        old_x + width + offset_x + xi as i64
                    };
                    let prior_y = y;
                    let index = self.offset_index((x, y)).expect("Out of bounds.");
                    let old_value = self.cells[index].take();
                    let new_value = reload((prior_x, prior_y), (x, y), old_value);
                    self.cells[index] = new_value;
                }
            }
            // The top/bottom partition
            for (iy, y) in new_y_range.clone().enumerate() {
                for (ix, x) in new_y_range_x_range.clone().enumerate() {
                    let prior_x = x;
                    let prior_y = if offset_y >= 0 {
                        old_y + iy as i64
                    } else {
                        old_y + height + offset_y + iy as i64
                    };
                    let index = self.offset_index((x, y)).expect("Out of bounds.");
                    let old_value = self.cells[index].take();
                    let new_value = reload((prior_x, prior_y), (x, y), old_value);
                    self.cells[index] = new_value;
                }
            }
            // The corner partition
            for (iy, y) in new_y_range.clone().enumerate() {
                for (ix, x) in new_x_range.clone().enumerate() {
                    let prior_x = if offset_x >= 0 {
                        old_x + ix as i64
                    } else {
                        old_x + width + offset_x + ix as i64
                    };
                    let prior_y = if offset_y >= 0 {
                        old_y + iy as i64
                    } else {
                        old_y + height + offset_y + iy as i64
                    };
                    let index = self.offset_index((x, y)).expect("Out of bounds.");
                    let old_value = self.cells[index].take();
                    let new_value = reload((prior_x, prior_y), (x, y), old_value);
                    self.cells[index] = new_value;
                }
            }
        } else {
            // Reload everything
            for (yi, y) in (new_y..new_y + height).enumerate() {
                for (xi, x) in (new_x..new_x + width).enumerate() {
                    let prior_x = old_x + xi as i64;
                    let prior_y = old_y + yi as i64;
                    let index = self.offset_index((x, y)).expect("Out of bounds.");
                    let old_value = self.cells[index].take();
                    let new_value = reload((prior_x, prior_y), (x, y), old_value);
                    self.cells[index] = new_value;
                }
            }
        }
    }

    pub fn iter<'a>(&'a self) -> RollingGridIterator<'a,W,H,T> {
        RollingGridIterator {
            offset: self.offset,
            grid: self,
            index: 0
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> RollingGridMutIterator<'a,W,H,T> {
        RollingGridMutIterator {
            offset: self.offset,
            grid: self,
            index: 0,
        }
    }
}

pub struct RollingGridIterator<'a, const W: usize, const H: usize, T> {
    grid: &'a RollingGrid<W,H,T>,
    offset: (i64, i64),
    index: usize,
}

impl<'a, const W: usize, const H: usize, T> Iterator for RollingGridIterator<'a, W, H, T> {
    type Item = ((i64, i64), Option<&'a T>);
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.grid.len() - self.index;
        (size, Some(size))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let minx = self.grid.offset.0;
        let miny = self.grid.offset.1;
        let maxx = minx + W as i64;
        let maxy = miny + H as i64;
        if !(minx..maxx).contains(&self.offset.0)
        || !(miny..maxy).contains(&self.offset.1) {
            return None;
        }
        let index = self.grid.offset_index(self.offset)?;
        let offset = self.offset;
        self.index += 1;
        self.offset.0 += 1;
        if self.offset.0 >= maxx {
            self.offset.0 = minx;
            self.offset.1 += 1;
        }
        unsafe {
            let cells_ptr = self.grid.cells.as_ptr();
            let cell_ptr = cells_ptr.add(index);
            if let Some(cell) = &*cell_ptr {
                Some((offset, Some(cell)))
            } else {
                Some((offset, None))
            }
        }
    }
    
}

pub struct RollingGridMutIterator<'a,const W: usize, const H: usize, T> {
    grid: &'a mut RollingGrid<W,H,T>,
    offset: (i64, i64),
    index: usize,
}

impl<'a, const W: usize, const H: usize, T> Iterator for RollingGridMutIterator<'a, W, H, T> {
    type Item = ((i64, i64), Option<&'a mut T>);
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.grid.len() - self.index;
        (size, Some(size))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let minx = self.grid.offset.0;
        let miny = self.grid.offset.1;
        let maxx = minx + W as i64;
        let maxy = miny + H as i64;
        if !(minx..maxx).contains(&self.offset.0)
        || !(miny..maxy).contains(&self.offset.1) {
            return None;
        }
        let index = self.grid.offset_index(self.offset)?;
        let offset = self.offset;
        self.index += 1;
        self.offset.0 += 1;
        if self.offset.0 >= maxx {
            self.offset.0 = minx;
            self.offset.1 += 1;
        }
        let cells_ptr = self.grid.cells.as_mut_ptr();
        unsafe {
            let cell_ptr = cells_ptr.add(index);
            if let Some(cell) = &mut *cell_ptr {
                Some((offset, Some(cell)))
            } else {
                Some((offset, None))
            }
        }
    }
    
}

impl<const W: usize, const H: usize, T> From<Grid<W,H,T>> for RollingGrid<W,H,T> {
    fn from(value: Grid<W,H,T>) -> Self {
        Self {
            cells: value.cells.into_iter().map(Option::Some).collect(),
            offset: (0, 0),
            roll: (0, 0),
        }
    }
}

mod tests {
    use itertools::Itertools;

    use super::*;
    #[test]
    fn rolling_grid_test() {
        let mut grid = RollingGrid::<8, 9, (i64, i64)>::new_with_init((0, 0), |(x, y)| {
            Some((x, y))
        });
        macro_rules! check_grid {
            ($x:expr, $y:expr) => {
                if let Some(&pos) = grid.get(($x, $y)) {
                    assert_eq!(pos, ($x, $y));
                } else {
                    panic!("Doesn't exist.");
                }
            };
        }
        check_grid!(3, 2);
        fn reload(old: (i64, i64), new: (i64, i64), old_value: Option<(i64, i64)>) -> Option<(i64, i64)> {
            Some(new)
        }
        grid.translate((1, 2), reload);
        check_grid!(8, 9);
        grid.translate((8, 8), reload);
        check_grid!(16, 17);
        grid.translate((-1, -1), reload);
        check_grid!(8, 9);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        grid.translate((-5, -5), reload);
        let (x, y) = grid.offset;
        for y in (y..y + grid.height() as i64) {
            for x in (x..x + grid.width() as i64) {
                check_grid!(x, y);
            }
        }
        println!("Offset: ({x}, {y})");
        let mut count = 0;
        grid.iter_mut().for_each(|((x, y), element)| {
            count += 1;
            if let Some(pos) = element {
                *pos = (0, 0);
            }
        });
        assert_eq!(count, grid.len());
        let mut count = 0;
        grid.iter().for_each(|((x, y), element)| {
            count += 1;
            if let Some(&element) = element {
                assert_eq!(element, (0, 0));
            } else {
                panic!("Element was None.");
            }
        });
        assert_eq!(count, grid.len());
        let grid_vec = grid.iter().map(|_| {
            0u8
        }).collect_vec();
        assert_eq!(grid_vec.len(), grid_vec.capacity());
    }
    
    #[test]
    fn grid_test() {
        for i in (-10..-20) {
            println!("{i}");
        }
        let mut grid = Grid::<32, 32, u8>::new();
        grid[(0, 1)] = 13;
        assert_eq!(grid[(0, 1)], 13);
        grid[(31, 31)] = 255;
        assert_eq!(grid[(31, 31)], 255);
        let mut grid = Grid8x8Bitmask::new();
        grid.set(1, 2, true);
        assert!(grid.get(1, 2));
        let grid: Grid<8, 8, bool> = grid.to_grid();
        if let Some(value) = grid.get(1, 2) {
            assert!(*value);
        } else {
            panic!()
        }
        let grid: Grid8x8Bitmask = grid.to_bitmask();
        assert!(grid.get(1, 2));
        let mut grid = Grid::<4, 4, (usize, usize)>::new_with_init(|pos| {
            pos
        });
        let mut count = 0;
        grid.iter().for_each(|((x, y), &cur)| {
            count += 1;
            assert_eq!((x, y), cur);
        });
        assert_eq!(count, grid.len());
        let mut count = 0;
        grid.iter_mut().for_each(|((x, y), cell)| {
            count += 1;
            *cell = (x + 1, y + 1);
        });
        assert_eq!(count, grid.len());
        let mut count = 0;
        grid.iter().for_each(|((x, y), &cur)| {
            count += 1;
            assert_eq!((x + 1, y + 1), cur);
        });
        assert_eq!(count, grid.len());
    } 
}