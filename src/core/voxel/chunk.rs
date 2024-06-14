pub struct Section {
    blocks: Box<[u32]>
}

impl Section {
    pub fn new() -> Self {
        Self {
            blocks: (0..4096).map(|_| 0).collect()
        }
    }

    // pub fn get(&self, x: usize, y: usize) -> Option<u32> {

    // }
}

#[test]
fn quick() {
    struct Dog {
        name: String
    }
    impl Dog {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_owned()
            }
        }
    }
    let mut cells = [Dog::new("Ralph"), Dog::new("Jimbob")];
    let ralph = &cells[0];
    let jimbob = &cells[1];
    
}