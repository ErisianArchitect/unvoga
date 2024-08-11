// level_of_detail.rs

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LOD {
    #[default]
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}