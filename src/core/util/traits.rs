pub trait StrToOwned {
    fn str_to_owned(self) -> String;
}

impl StrToOwned for String {
    fn str_to_owned(self) -> String {
        self
    }
}

impl StrToOwned for &String {
    fn str_to_owned(self) -> String {
        self.to_owned()
    }
}

impl StrToOwned for &str {
    fn str_to_owned(self) -> String {
        self.to_owned()
    }
}