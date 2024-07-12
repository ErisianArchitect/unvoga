use std::any::Any;

pub trait BlockData: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[test]
pub fn test() {
    struct Data(&'static str);
    impl BlockData for Data {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    let data: Box<dyn BlockData> = Box::new(Data("Test"));
    let dat: Option<&Data> = data.as_any().downcast_ref();
    if let Some(dat) = dat {
        println!("{}", dat.0);
    }
}