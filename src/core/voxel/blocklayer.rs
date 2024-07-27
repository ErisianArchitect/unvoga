#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum BlockLayer {
    #[default]
    Base = 0,
    Transparent = 1,
    Other(u16) = 3,
}

impl BlockLayer {
    pub const fn transparent(self) -> bool {
        matches!(self, BlockLayer::Transparent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ord_test() {
        let layer0 = BlockLayer::Base;
        let layer1 = BlockLayer::Transparent;
        let layer2 = BlockLayer::Other(0);
        let layer3 = BlockLayer::Other(1);
        assert!(layer0 < layer1);
        assert!(layer1 < layer2);
        assert!(layer2 < layer3);
    }
}