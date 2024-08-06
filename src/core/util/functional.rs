use std::marker::PhantomData;

pub fn invoke<R, F: FnMut() -> R>(mut f: F) -> R {
    f()
}

pub trait Pipe: Sized {
    fn pipe_into<R, F: FnOnce(Self) -> R>(self, f: F) -> R;
}

impl<T> Pipe for T {
    fn pipe_into<R, F: FnOnce(Self) -> R>(self, f: F) -> R {
        f(self)
    }
}

pub trait Pipeline {
    type Input;
    type Output;
    fn eval(self, input: Self::Input) -> Self::Output;
}

struct PiecePipeMut<T, R, F: FnMut(T) -> R> {
    f: F,
    _t: PhantomData<T>,
    _r: PhantomData<R>,
}

struct PiecePipe<T, R, F: Fn(T) -> R> {
    f: F,
    _t: PhantomData<T>,
    _r: PhantomData<R>,
}

impl<T, R, F: FnMut(T) -> R> From<F> for PiecePipeMut<T, R, F> {
    fn from(value: F) -> Self {
        Self {
            f: value,
            _t: PhantomData::default(),
            _r: PhantomData::default(),
        }
    }
}

impl<T, R, F: Fn(T) -> R> From<F> for PiecePipe<T, R, F> {
    fn from(value: F) -> Self {
        Self {
            f: value,
            _t: PhantomData::default(),
            _r: PhantomData::default(),
        }
    }
}

impl<T, R, F: FnMut(T) -> R> Pipeline for PiecePipeMut<T, R, F> {
    type Input = T;
    type Output = R;
    fn eval(mut self, input: Self::Input) -> Self::Output {
        (self.f)(input)
    }
}

impl<T, R, F: Fn(T) -> R> Pipeline for PiecePipe<T, R, F> {
    type Input = T;
    type Output = R;
    fn eval(mut self, input: Self::Input) -> Self::Output {
        (self.f)(input)
    }
}

impl<T, R, F: Fn(T) -> R> Pipeline for &PiecePipe<T, R, F> {
    type Input = T;
    type Output = R;
    fn eval(self, input: Self::Input) -> Self::Output {
        (self.f)(input)
    }
}

impl<T, R, F: FnMut(T) -> R> Pipeline for &mut PiecePipeMut<T, R, F> {
    type Input = T;
    type Output = R;
    fn eval(self, input: Self::Input) -> Self::Output {
        (self.f)(input)
    }
}

impl<T0, R0, F0: Fn(T0) -> R0, R1, F1: Fn(R0) -> R1> Pipeline for (PiecePipe<T0, R0, F0>, PiecePipe<R0, R1, F1>) {
    type Input = T0;
    type Output = R1;
    fn eval(mut self, input: Self::Input) -> Self::Output {
        (self.1.f)((self.0.f)(input))
    }
}

pub struct PipelineBuilder {

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipe_test() {
        (1, 2, 3)
            .pipe_into(|(a, b, c)| a + b + c)
            .pipe_into(|n| n * n)
            .pipe_into(|n| println!("n: {n}"));
        /*
        PipelineBuilder::new(|n| (n * n, n + n))
            .
        */
    }
}