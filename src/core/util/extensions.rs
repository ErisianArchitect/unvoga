use std::ops::Range;

use crate::for_each_int_type;

pub trait SwapVal {
    #[inline(always)]
    fn swap(&mut self, swap: Self) -> Self;
}

impl<T> SwapVal for T {
    #[inline(always)]
    fn swap(&mut self, swap: Self) -> Self {
        let mut swap = swap;
        std::mem::swap(self, &mut swap);
        swap
    }
}

pub trait OptionExtension<T> {
    #[inline(always)]
    fn then<F: FnOnce(T)>(self, then: F);
}

impl<T> OptionExtension<T> for Option<T> {
    #[inline(always)]
    fn then<F: FnOnce(T)>(self, then: F) {
        if let Some(value) = self {
            then(value);
        }
    }
}

pub trait BoolExtension: 'static {
    #[inline(always)]
    fn choose<T>(self, _true: T, _false: T) -> T;
    #[inline(always)]
    fn invert(&mut self) -> Self;
    #[inline(always)]
    fn some<T>(self, some: T) -> Option<T>;
    #[inline(always)]
    fn some_else<T>(self, some: T) -> Option<T>;
    #[inline(always)]
    fn if_<F: Fn()>(self, _if: F);
    #[inline(always)]
    fn if_else<R, If: Fn() -> R, Else: Fn() -> R>(self, _if: If, _else: Else) -> R;
}

impl BoolExtension for bool {
    /// Choose a truth value or a false value.
    #[inline(always)]
    fn choose<T>(self, _true: T, _false: T) -> T {
        if self {
            _true
        } else {
            _false
        }
    }

    /// Inverts the value of the boolean.
    #[inline(always)]
    fn invert(&mut self) -> Self {
        *self = !*self;
        *self
    }

    /// Returns `Some(some)` if true.
    #[inline(always)]
    fn some<T>(self, some: T) -> Option<T> {
        self.choose(Some(some), None)
    }

    /// Returns `Some(some)` if false.
    #[inline(always)]
    fn some_else<T>(self, some: T) -> Option<T> {
        self.choose(None, Some(some))
    }

    #[inline(always)]
    fn if_<F: Fn()>(self, _if: F) {
        if self {
            _if();
        }
    }

    /// Like `if-else`, but with closures!
    #[inline(always)]
    fn if_else<R, If: Fn() -> R, Else: Fn() -> R>(self, _if: If, _else: Else) -> R {
        if self {
            _if()
        } else {
            _else()
        }
    }
}

pub trait NumIter: Sized + Copy {
    #[inline(always)]
    fn iter(self) -> Range<Self>;
    #[inline(always)]
    fn iter_to(self, end: Self) -> Range<Self>;
}

macro_rules! num_iter_impls {
    ($type:ty) => {
        impl NumIter for $type {
            #[inline(always)]
            fn iter(self) -> Range<Self> {
                0..self
            }

            #[inline(always)]
            fn iter_to(self, end: Self) -> Range<Self> {
                self..end
            }
        }
    };
}

for_each_int_type!(num_iter_impls);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn num_iter_test() {
        30i32.iter_to(32).for_each(|i| {
            println!("{i}");
        });
    }
}