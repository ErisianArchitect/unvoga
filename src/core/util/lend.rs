pub struct Lend<T> {
    value: Option<T>,
}

impl<T> Lend<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value)
        }
    }

    pub fn lend(&mut self) -> T {
        let Some(value) = self.value.take() else {
            panic!("Value already taken!");
        };
        value
    }

    pub fn give(&mut self, value: T) {
        if self.value.is_some() {
            panic!("Value already present.");
        }
        self.value = Some(value);
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