pub trait SwapVal {
    fn swap_val(&mut self, swap: Self) -> Self;
}

impl<T> SwapVal for T {
    fn swap_val(&mut self, swap: Self) -> Self {
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