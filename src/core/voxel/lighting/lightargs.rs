/// Filter and Emit packed together.
/// First 4 bits are Filter, second 4 bits are Emit.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightArgs(u8);

impl LightArgs {
    
    pub fn new(filter: u8, emit: u8) -> LightArgs {
        let filter = filter.min(15);
        let emit = emit.min(15);
        LightArgs(filter | emit << 4)
    }

    /// Clamps value to 15 if greater.
    
    pub fn set_filter(&mut self, filter: u8) {
        self.0 = (self.0 & 0xF0) | filter.min(15);
    }

    
    pub fn filter(self) -> u8 {
        self.0 & 0xF
    }

    /// Clamps value to 15 if greater.
    
    pub fn set_emit(&mut self, emit: u8) {
        self.0 = (self.0 & 0xF) | (emit.min(15) << 4);
    }

    
    pub fn emit(self) -> u8 {
        self.0 >> 4
    }
}