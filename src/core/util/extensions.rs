use std::ops::Range;

use crate::for_each_int_type;

pub trait SwapVal {
    
    fn swap(&mut self, swap: Self) -> Self;
}

impl<T> SwapVal for T {
    
    fn swap(&mut self, swap: Self) -> Self {
        let mut swap = swap;
        std::mem::swap(self, &mut swap);
        swap
    }
}

pub trait OptionExtension<T> {
    
    fn then<F: FnOnce(T)>(self, then: F);
}

impl<T> OptionExtension<T> for Option<T> {
    
    fn then<F: FnOnce(T)>(self, then: F) {
        if let Some(value) = self {
            then(value);
        }
    }
}

pub trait BoolExtension: 'static {
    
    fn choose<T>(self, _true: T, _false: T) -> T;
    
    fn invert(&mut self) -> Self;
    
    fn some<T>(self, some: T) -> Option<T>;
    
    fn some_else<T>(self, some: T) -> Option<T>;
    
    fn if_<F: Fn()>(self, _if: F);
    
    fn if_else<R, If: Fn() -> R, Else: Fn() -> R>(self, _if: If, _else: Else) -> R;
}

impl BoolExtension for bool {
    /// Choose a truth value or a false value.
    
    fn choose<T>(self, _true: T, _false: T) -> T {
        if self {
            _true
        } else {
            _false
        }
    }

    /// Inverts the value of the boolean.
    
    fn invert(&mut self) -> Self {
        *self = !*self;
        *self
    }

    /// Returns `Some(some)` if true.
    
    fn some<T>(self, some: T) -> Option<T> {
        self.choose(Some(some), None)
    }

    /// Returns `Some(some)` if false.
    
    fn some_else<T>(self, some: T) -> Option<T> {
        self.choose(None, Some(some))
    }

    
    fn if_<F: Fn()>(self, _if: F) {
        if self {
            _if();
        }
    }

    /// Like `if-else`, but with closures!
    
    fn if_else<R, If: Fn() -> R, Else: Fn() -> R>(self, _if: If, _else: Else) -> R {
        if self {
            _if()
        } else {
            _else()
        }
    }
}

pub trait NumIter: Sized + Copy {
    
    fn iter(self) -> Range<Self>;
    
    fn iter_to(self, end: Self) -> Range<Self>;
}

macro_rules! num_iter_impls {
    ($type:ty) => {
        impl NumIter for $type {
            
            fn iter(self) -> Range<Self> {
                0..self
            }

            
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