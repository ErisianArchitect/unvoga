

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectorOffset(u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockSize(u8);

impl BlockSize {
    #[inline(always)]
    pub const fn new(block_count: u8, exponent: u8) -> Self {
        if block_count > 0b11111 {
            panic!("Block Count greater than 31.");
        }
        if exponent > 0b111 {
            panic!("Exponent greater than 7");
        }
        Self(block_count | exponent << 5)
    }

    #[inline(always)]
    pub const fn block_count(self) -> u64 {
        let block_count = self.0 & 0b11111;
        let exponent = self.0 >> 5;
        block_size_notation(block_count as u64, exponent as u32, 5)
    }
}

#[inline(always)]
pub const fn block_size_notation(block_count: u64, exponent: u32, bit_size: u32) -> u64 {
    let max_block_size = 2u64.pow(bit_size)-1;
    let spacer1 = (2u64.pow(exponent) - 1) * max_block_size;
    let spacer2 = if exponent > 0 {
        2u64.pow(exponent)
    } else {
        0
    };
    block_count * 2u64.pow(exponent) + spacer1 + spacer2
}

#[test]
fn bsn_test() {
    let block_size = BlockSize::new(31, 7);
    println!("Block Count: {}", block_size.block_count());
}

// bit size of 5 is the sweet spot
/*
def block_size_notation(block_count: int, exponent: int = 0, bit_size: int = 4)->int:
    max_block_size = 2**bit_size-1
    spacer1 = ((2**exponent) - 1) * max_block_size
    spacer2 = 2**exponent if exponent > 0 else 0
    return ((block_count * (2**exponent)) + (spacer1 + spacer2))
*/