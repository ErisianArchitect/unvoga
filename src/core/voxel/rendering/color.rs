use bytemuck::NoUninit;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self{r,g,b}
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Rgba {
    
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self{r,g,b,a}
    }
}