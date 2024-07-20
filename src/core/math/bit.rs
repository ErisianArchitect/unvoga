use bytemuck::NoUninit;

use std::ops::{
    Range,
    RangeBounds,
};

use crate::for_each_int_type;

pub trait BitFlags: Sized + Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash {
    fn get(self, index: u32) -> bool;
    fn set(&mut self, index: u32, value: bool) -> bool;
    fn iter(self) -> impl Iterator<Item = bool>;
    const BIT_SIZE: u32;
}

macro_rules! bitflags_impls {
    ($type:ident($inner_type:ty)) => {
        #[repr(C)]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit)]
        pub struct $type(pub $inner_type);

        impl std::ops::BitOr<$type> for $type {
            type Output = Self;
            
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
        
        impl std::ops::BitOr<$inner_type> for $type {
            type Output = Self;
            
            fn bitor(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 | rhs)
            }
        }
        
        impl std::ops::BitAnd<$type> for $type {
            type Output = Self;
            
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        
        impl std::ops::BitAnd<$inner_type> for $type {
            type Output = Self;
            
            fn bitand(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 & rhs)
            }
        }
        
        impl std::ops::Sub<$type> for $type {
            type Output = Self;
            
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 & !rhs.0)
            }
        }
        
        impl std::ops::Sub<$inner_type> for $type {
            type Output = Self;
            
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
    ($($type:ident($inner_type:ty);)*) => {
        $(
            bitflags_impls!{$type($inner_type)}
        )*
    };
}

bitflags_impls!(
    BitFlags8(u8);
    BitFlags16(u16);
    BitFlags32(u32);
    BitFlags64(u64);
    BitFlags128(u128);
);

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

pub trait BitSize {
    const BITSIZE: usize;
}

pub trait BitLength {
    fn bit_length(self) -> u32;
}

macro_rules! bit_length {
    ($type:ty) => {
        impl BitLength for $type {
            
            fn bit_length(self) -> u32 {
                const BIT_WIDTH: u32 = (std::mem::size_of::<$type>() * 8) as u32;
                BIT_WIDTH - self.leading_zeros()
            }
        }
    };
}

for_each_int_type!(bit_length);

pub trait ShiftIndex: Copy {
    /// A `u32` value that represents an index that a `1` bit can be shifted to.
    /// This simply converts the value to u32.
    
    fn shift_index(self) -> u32;
}

macro_rules! __shiftindex_impls {
    ($type:ty) => {
        impl ShiftIndex for $type {
            
            fn shift_index(self) -> u32 {
                self as u32
            }
        }
    };
}

for_each_int_type!(__shiftindex_impls);

macro_rules! __bitsize_impls {
    ($type:ty) => {
        impl BitSize for $type {
            const BITSIZE: usize = std::mem::size_of::<$type>() * 8;
        }
    };
}

for_each_int_type!(__bitsize_impls);

pub trait SetBit {
    
    fn set_bit<I: ShiftIndex>(self, index: I, on: bool) -> Self;
    
    fn set_bitmask(self, mask: Range<u32>, value: Self) -> Self;
    
    fn bitmask_range(mask: Range<u32>) -> Self;
}

pub trait GetBit {
    
    fn get_bit<I: ShiftIndex>(self, index: I) -> bool;
    
    fn get_bitmask(self, mask: Range<u32>) -> Self;
}

pub trait InvertBit {
    
    fn invert_bit<I: ShiftIndex>(self, index: I) -> Self;
}

impl<T: GetBit + SetBit + Copy> InvertBit for T {
    
    fn invert_bit<I: ShiftIndex>(self, index: I) -> Self {
        let bit = self.get_bit(index);
        self.set_bit(index, !bit)
    }
}

macro_rules! __get_set_impl {
    ($type:ty) => {

        impl SetBit for $type {
            
            fn set_bit<I: ShiftIndex>(self, index: I, on: bool) -> Self {
                if let (mask, false) = (1 as $type).overflowing_shl(index.shift_index()) {
                    if on {
                        self | mask
                    } else {
                        self & !mask
                    }
                } else {
                    self
                }
            }

            
            fn set_bitmask(self, mask: Range<u32>, value: Self) -> Self {
                let mask_len = mask.len();
                let size_mask = ((1 as Self) << mask_len)-1;
                let bitmask = size_mask << mask.start;
                let delete = self & !bitmask;
                let value = value & size_mask;
                delete | value << mask.start
            }

            
            fn bitmask_range(range: Range<u32>) -> Self {
                (((1 as $type) << range.len()) - 1) << range.start
            }
        }

        impl GetBit for $type {
            
            fn get_bit<I: ShiftIndex>(self, index: I) -> bool {
                if let (mask, false) = (1 as $type).overflowing_shl(index.shift_index()) {
                    (self & mask) != 0
                } else {
                    false
                }
            }

            
            fn get_bitmask(self, mask: Range<u32>) -> Self {
                let mask_len = mask.len();
                let bitmask = (((1 as Self) << mask_len)-1) << mask.start;
                (self & bitmask) >> mask.start
            }
        }

    };
}

crate::for_each_int_type!(__get_set_impl);

/// To allow polymorphism for iterators of different integer types or references to integer types.
pub trait MoveBitsIteratorItem {
    fn translate(self) -> usize;
}

pub trait MoveBits: Sized {
    fn move_bits<T: MoveBitsIteratorItem, It: IntoIterator<Item = T>>(self, new_indices: It) -> Self;
    /// Much like move_bits, but takes indices in reverse order. This is useful if you want to have the
    /// indices laid out more naturally from right to left.
    fn move_bits_rev<T: MoveBitsIteratorItem, It: IntoIterator<Item = T>>(self, new_indices: It) -> Self
    where It::IntoIter: DoubleEndedIterator {
        self.move_bits(new_indices.into_iter().rev())
    }
}

macro_rules! __movebits_impls {
    ($type:ty) => {
        impl MoveBitsIteratorItem for $type {
            fn translate(self) -> usize {
                self as usize
            }
        }

        impl MoveBitsIteratorItem for &$type {
            fn translate(self) -> usize {
                *self as usize
            }
        }
    };
}

for_each_int_type!(__movebits_impls);

impl<T: BitSize + GetBit + SetBit + Copy> MoveBits for T {
    fn move_bits<I: MoveBitsIteratorItem, It: IntoIterator<Item = I>>(self, source_indices: It) -> Self {
        source_indices.into_iter()
            .map(I::translate)
            .enumerate()
            .take(Self::BITSIZE)
            .fold(self, |value, (index, swap_index)| {
                let on = value.get_bit(swap_index);
                value.set_bit(index, on)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn bitmask_test() {
        // 4..7
        let value = 0b11100000111;
        let new = value.set_bitmask(4..7, 0b101);
        let mask = new.get_bitmask(2..9);
        assert_eq!(mask, 0b1010101);

    }
}