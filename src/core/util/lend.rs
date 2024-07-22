use std::borrow::BorrowMut;

pub struct Lend<T> {
    value: Option<T>,
}

impl<T> Lend<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value)
        }
    }

    pub fn lend(&mut self, reason: &'static str) -> Borrowed<T> {
        let Some(value) = self.value.take() else {
            panic!("Value already taken!");
        };
        Borrowed(Some(value), reason)
    }

    pub fn give(&mut self, value: Borrowed<T>) {
        if self.value.is_some() {
            panic!("Value already present.");
        }
        self.value = Some(value.take());
    }
}

impl<T> std::ops::Deref for Lend<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let Some(value) = &self.value else {
            panic!("Value wasn't present");
        };
        value
    }
}

impl<T> std::ops::DerefMut for Lend<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Some(value) = &mut self.value else {
            panic!("Value wasn't present");
        };
        value
    }
}

pub struct Borrowed<T>(Option<T>,&'static str);

impl<T> Borrowed<T> {
    fn take(mut self) -> T {
        self.0.take().expect("Invalid Borrowed value")
    }
}

impl<T> std::ops::Deref for Borrowed<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let Some(value) = &self.0 else {
            panic!("Invalid Borrowed value");
        };
        value
    }
}

impl<T> std::ops::DerefMut for Borrowed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Some(value) = &mut self.0 else {
            panic!("Invalid Borrowed value");
        };
        value
    }
}

impl<T> Drop for Borrowed<T> {
    fn drop(&mut self) {
        if self.0.is_some() {
            panic!("Borrowed data from Lend structure but did not return it: {:?}", self.1);
        }
    }
}