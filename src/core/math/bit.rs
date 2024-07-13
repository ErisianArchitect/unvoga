use bytemuck::NoUninit;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct BitFlags8(pub u8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct BitFlags16(pub u16);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct BitFlags32(pub u32);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct BitFlags64(pub u64);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
pub struct BitFlags128(pub u128);

pub trait BitFlags: Sized + Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash {
    fn get(self, index: u32) -> bool;
    fn set(&mut self, index: u32, value: bool) -> bool;
    fn iter(self) -> impl Iterator<Item = bool>;
    const BIT_SIZE: u32;
}

macro_rules! bitflags_impl {
    (<$type:ty>($inner_type:ty)) => {
        impl std::ops::BitOr<$type> for $type {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
        
        impl std::ops::BitOr<$inner_type> for $type {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 | rhs)
            }
        }
        
        impl std::ops::BitAnd<$type> for $type {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        
        impl std::ops::BitAnd<$inner_type> for $type {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 & rhs)
            }
        }
        
        impl std::ops::Sub<$type> for $type {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 & !rhs.0)
            }
        }
        
        impl std::ops::Sub<$inner_type> for $type {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 & !rhs)
            }
        }

        impl std::ops::Index<u32> for $type {
            type Output = bool;
            fn index(&self, index: u32) -> &Self::Output {
                const FALSE_TRUE: [bool; 2] = [false, true];
                let index = ((self.0 & (1 << index)) != 0) as usize;
                &FALSE_TRUE[index]
            }
        }

        impl BitFlags for $type {
            const BIT_SIZE: u32 = (std::mem::size_of::<Self>() * 8) as u32;
            fn get(self, index: u32) -> bool {
                (self.0 & (1 << index)) != 0
            }
        
            fn set(&mut self, index: u32, value: bool) -> bool {
                let old = (self.0 & (1 << index)) != 0;
                if value {
                    self.0 = self.0 | (1 << index);
                } else {
                    self.0 = self.0 & !(1 << index);
                }
                old
            }
        
            fn iter(self) -> impl Iterator<Item = bool> {
                (0..Self::BIT_SIZE).map(move |i| self.get(i))
            }
        }
    };
}

bitflags_impl!(<BitFlags8>(u8));
bitflags_impl!(<BitFlags16>(u16));
bitflags_impl!(<BitFlags32>(u32));
bitflags_impl!(<BitFlags64>(u64));
bitflags_impl!(<BitFlags128>(u128));

impl std::fmt::Display for BitFlags8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitFlags8({:08b})", self.0)
    }
}

impl std::fmt::Display for BitFlags16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitFlags16(")?;
        let low = self.0 & 0xFF;
        let high = self.0 >> 8;
        write!(f, "{high:08b} {low:08b}")?;
        write!(f, ")")
    }
}

impl std::fmt::Display for BitFlags32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitFlags32(")?;
        let _0 = self.0 & 0xFF;
        let _1 = self.0 >> 8 & 0xFF;
        let _2 = self.0 >> 16 & 0xFF;
        let _3 = self.0 >> 24 & 0xFF;
        write!(f, "{_3:08b} {_2:08b} {_1:08b} {_0:08b})")
    }
}

impl std::fmt::Display for BitFlags64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitFlags64(")?;
        let _0 = self.0 & 0xFF;
        let _1 = self.0 >> 8 & 0xFF;
        let _2 = self.0 >> 16 & 0xFF;
        let _3 = self.0 >> 24 & 0xFF;
        let _4 = self.0 >> 32 & 0xFF;
        let _5 = self.0 >> 40 & 0xFF;
        let _6 = self.0 >> 48 & 0xFF;
        let _7 = self.0 >> 56 & 0xFF;
        write!(f, "{_7:08b} {_6:08b} {_5:08b} {_4:08b} {_3:08b} {_2:08b} {_1:08b} {_0:08b})")
    }
}

impl std::fmt::Display for BitFlags128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitFlags128(")?;

        let _0  = self.0 >> ( 0 * 8) & 0xFF;
        let _1  = self.0 >> ( 1 * 8) & 0xFF;
        let _2  = self.0 >> ( 2 * 8) & 0xFF;
        let _3  = self.0 >> ( 3 * 8) & 0xFF;

        let _4  = self.0 >> ( 4 * 8) & 0xFF;
        let _5  = self.0 >> ( 5 * 8) & 0xFF;
        let _6  = self.0 >> ( 6 * 8) & 0xFF;
        let _7  = self.0 >> ( 7 * 8) & 0xFF;

        let _8  = self.0 >> ( 8 * 8) & 0xFF;
        let _9  = self.0 >> ( 9 * 8) & 0xFF;
        let _10 = self.0 >> (10 * 8) & 0xFF;
        let _11 = self.0 >> (11 * 8) & 0xFF;

        let _12 = self.0 >> (12 * 8) & 0xFF;
        let _13 = self.0 >> (13 * 8) & 0xFF;
        let _14 = self.0 >> (14 * 8) & 0xFF;
        let _15 = self.0 >> (15 * 8) & 0xFF;

        write!(f, "{_15:08b} {_14:08b} {_13:08b} {_12:08b} {_11:08b} {_10:08b} {_9:08b} {_8:08b} {_7:08b} {_6:08b} {_5:08b} {_4:08b} {_3:08b} {_2:08b} {_1:08b} {_0:08b})")
    }
}

#[test]
fn bit_test() {
    let bits = BitFlags8(0b1010101);
    for (i, bit) in bits.iter().enumerate() {
        println!("{i} = {bit}");
    }
}