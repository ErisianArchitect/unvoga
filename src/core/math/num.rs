#![allow(unused)]
use crate::for_each_int_type;

pub trait Zero {
    const ZERO: Self;
}

pub trait One {
    const ONE: Self;
}

pub trait NegOne {
    const NEG1: Self;
}

pub trait Two {
    const TWO: Self;
}

pub trait NegTwo {
    const NEG2: Self;
}

pub trait Min {
    const MIN: Self;
}

pub trait Max {
    const MAX: Self;
}

pub trait Add {
    fn add(self, rhs: Self) -> Self;
}

pub trait Sub {
    fn sub(self, rhs: Self) -> Self;
}

pub trait Mul {
    fn mul(self, rhs: Self) -> Self;
}

pub trait Div {
    fn div(self, rhs: Self) -> Self;
}

pub trait Neg {
    fn neg(self) -> Self;
}

pub trait Not {
    fn not(self) -> Self;
}

pub trait Rem {
    fn rem(self, rhs: Self) -> Self;
}

pub trait RemEuclid {
    fn rem_euclid(self, rhs: Self) -> Self;
}

pub trait LeadingZeros {
    fn leading_zeros(self) -> u32;
}

pub trait TrailingZeros {
    fn trailing_zeros(self) -> u32;
}

pub trait LeadingOnes {
    fn leading_ones(self) -> u32;
}

pub trait TrailingOnes {
    fn trailing_ones(self) -> u32;
}

pub trait Shl {
    fn shl(self, rhs: Self) -> Self;
}

pub trait Shr {
    fn shr(self, rhs: Self) -> Self;
}

pub trait BitAnd {
    fn bitand(self, rhs: Self) -> Self;
}

pub trait BitOr {
    fn bitor(self, rhs: Self) -> Self;
}

pub trait BitXor {
    fn bitxor(self, rhs: Self) -> Self;
}

pub trait UnsignedNum: 
    Zero
    + One
    + Two
    + Min
    + Max
    + Add
    + Sub
    + Mul
    + Div
    + Not
    + Rem
    + Shl
    + Shr
    + BitAnd
    + BitOr
    + BitXor
    + RemEuclid
    + LeadingZeros
    + TrailingZeros
    + LeadingOnes
    + TrailingOnes
    {}
pub trait SignedNum: UnsignedNum + Neg + NegOne + NegTwo {}

macro_rules! num_impls {
    ($type:ty) => {
        impl UnsignedNum for $type {}
        // impl SignedNum for $type {}
        impl Zero for $type {
            const ZERO: Self = 0;
        }

        impl One for $type {
            const ONE: Self = 1;
        }

        impl Two for $type {
            const TWO: Self = 2;
        }

        impl Min for $type {
            const MIN: Self = <$type>::MIN;
        }

        impl Max for $type {
            const MAX: Self = <$type>::MAX;
        }

        impl Add for $type {
            
            fn add(self, rhs: Self) -> Self {
                self + rhs
            }
        }

        impl Sub for $type {
            
            fn sub(self, rhs: Self) -> Self {
                self - rhs
            }
        }

        impl Mul for $type {
            
            fn mul(self, rhs: Self) -> Self {
                self * rhs
            }
        }

        impl Div for $type {
            
            fn div(self, rhs: Self) -> Self {
                self / rhs
            }
        }

        impl Not for $type {
            
            fn not(self) -> Self {
                !self
            }
        }

        impl Rem for $type {
            
            fn rem(self, rhs: Self) -> Self {
                self % rhs
            }
        }

        impl RemEuclid for $type {
            
            fn rem_euclid(self, rhs: Self) -> Self {
                <$type>::rem_euclid(self, rhs)
            }
        }

        impl LeadingZeros for $type {
            
            fn leading_zeros(self) -> u32 {
                <$type>::leading_zeros(self)
            }
        }

        impl TrailingZeros for $type {
            
            fn trailing_zeros(self) -> u32 {
                <$type>::trailing_zeros(self)
            }
        }

        impl LeadingOnes for $type {
            
            fn leading_ones(self) -> u32 {
                <$type>::leading_ones(self)
            }
        }

        impl TrailingOnes for $type {
            
            fn trailing_ones(self) -> u32 {
                <$type>::trailing_ones(self)
            }
        }

        impl Shl for $type {
            
            fn shl(self, rhs: Self) -> Self {
                self << rhs
            }
        }

        impl Shr for $type {
            
            fn shr(self, rhs: Self) -> Self {
                self >> rhs
            }
        }

        impl BitAnd for $type {
            
            fn bitand(self, rhs: Self) -> Self {
                self & rhs
            }
        }

        impl BitOr for $type {
            
            fn bitor(self, rhs: Self) -> Self {
                self | rhs
            }
        }

        impl BitXor for $type {
            
            fn bitxor(self, rhs: Self) -> Self {
                self ^ rhs
            }
        }
    };
}

macro_rules! neg_impls {
    ($type:ty) => {
        impl SignedNum for $type {}
        impl NegOne for $type {
            const NEG1: Self = -1;
        }

        impl NegTwo for $type {
            const NEG2: Self = -2;
        }

        impl Neg for $type {
            
            fn neg(self) -> Self {
                -self
            }
        }
    };
}

for_each_int_type!(num_impls);

for_each_int_type!(neg_impls; signed);

#[cfg(test)]
mod testing_sandbox {
    use super::*;
    #[test]
    fn sandbox() {
        println!("i32: {}", take_num(10i32, 5i32));
        println!("u32: {}", take_num(10u64, 5u64));
    }

    fn take_num<T: UnsignedNum>(left: T, right: T) -> T {
        left.add(right).sub(T::ONE)
    }
}