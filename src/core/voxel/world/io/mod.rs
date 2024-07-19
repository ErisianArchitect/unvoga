
use hashbrown::HashMap;

use crate::core::{io::*, voxel::blockstate::BlockState};
use crate::prelude::*;
use crate::core::error::*;

use std::io::{Read, Write};

/*
0b00111111
0b00001111
0b00000011
0b11111100 (3 bytes, not 4)*/
/// Reads 4096 [Occlusion]s from a reader as 3072 bytes.
// This function will likely want to be inlined
#[inline(always)]
pub fn read_section_occlusions<R: Read>(reader: &mut R) -> Result<Box<[Occlusion]>> {
    let mut bytes = [0u8; 6*4096/8];
    reader.read_exact(&mut bytes)?;
    let mut occlusions = (0..4096).map(|_| Occlusion::UNOCCLUDED).collect::<Box<[_]>>();
    let mut offset = 0;
    let mut occlusion_index = 0;
    while offset < bytes.len() {
        let a = bytes[offset+0];
        let b = bytes[offset+1];
        let c = bytes[offset+2];
        let first = a & 0b111111;
        let second = (a >> 6) | ((b & 0b1111) << 2);
        let third = (b >> 4) | ((c & 0b11) << 4);
        let fourth = c >> 2;
        occlusions[occlusion_index+0] = Occlusion(first);
        occlusions[occlusion_index+1] = Occlusion(second);
        occlusions[occlusion_index+2] = Occlusion(third);
        occlusions[occlusion_index+3] = Occlusion(fourth);
        offset += 3;
        occlusion_index += 4;
    }
    Ok(occlusions)
}
// This function will likely want to be inlined.
#[inline(always)]
pub fn write_section_occlusions<W: Write>(writer: &mut W, occlusions: &[Occlusion]) -> Result<u64> {
    assert_eq!(occlusions.len(), 4096, "Occlusions must be 4096 in length");
    let mut bytes = [0u8; 6*4096/8];
    let mut byte_i = 0usize;
    let mut occ_i = 0usize;
    while occ_i < 4096 {
        let o0 = occlusions[occ_i + 0].0;
        let o1 = occlusions[occ_i + 1].0;
        let o2 = occlusions[occ_i + 2].0;
        let o3 = occlusions[occ_i + 3].0;
        bytes[byte_i + 0] = o0 | (o1 << 6);
        bytes[byte_i + 1] = (o1 >> 2) | (o2 << 4);
        bytes[byte_i + 2] = (o2 >> 4) | (o3 << 2);
        occ_i += 4;
        byte_i += 3;
    }
    writer.write_all(&bytes)?;
    Ok(bytes.len() as u64)
}

pub fn read_section_blocks<R: Read>(reader: &mut R, blocks: &mut Option<Box<[Id]>>) -> Result<()> {
    todo!()
}

pub fn write_section_blocks<W: Write>(writer: &mut W, blocks: &mut Option<Box<[Id]>>) -> Result<u64> {
    let Some(blocks) = blocks else {
        // Empty chunk, so just write a null byte and return.
        return 0u8.write_to(writer);
    };
    // Map blocks to new ids.
    let mut id_map = HashMap::<Id, u16>::new();
    let mut id_counter = 0u16;
    let mut ids = Vec::new();
    for i in 0..4096 {
        let entry = id_map.entry(blocks[i]).or_insert(id_counter);
        if *entry == id_counter {
            id_counter += 1;
            ids.push(blocks[i]);
        }
    }
    let mut length = 0u64;
    if id_counter < 128 {
        let count = id_counter as u8;
        length += count.write_to(writer)?;
    } else {
        // Set the last bit so that I can mark the counter as two bytes.
        // Gotta make sure to reverse this for the reader.
        let add_bit = id_counter | 0b10000000_00000000;
        // We're assuming the write_to method uses big endian byte order, which it should.
        length += add_bit.write_to(writer)?;
    }
    length = ids.into_iter().try_fold(length, |length, id| {
        // Just because I want to make sure I don't somehow accidentally write the id.
        let blockstate: &BlockState = &*id;
        Result::Ok(length + blockstate.write_to(writer)?)
    })?;
    // We only need to write a single block to the block table, and then return
    // early because there's no need to write any indices. The whole section is
    // the same block id.
    if id_counter == 1 {
        return Ok(length);
    }
    // We can get the bit width by getting the next power of two and counting the trailing zeros.
    // thankfully Rust has functions for both of these operations.
    // 0b001101
    // 0b010000
    let bit_width = id_counter.next_power_of_two().trailing_zeros();
    let byte_count = (bit_width as u32  * 4096) / 8 + (bit_width % 8 != 0) as u32;
    let mut bytes = (0..byte_count).map(|_| 0u8).collect::<Box<_>>();
    struct BitWriter<'a> {
        // The bits that have been added (bit width is bit_width)
        accum: u8,
        accum_size: u32,
        bit_width: u32,
        bytes: &'a mut [u8],
        byte_index: usize,
    }
    impl<'a> BitWriter<'a> {
        #[inline(always)]
        fn push_byte(&mut self, byte: u8) {
            self.bytes[self.byte_index] = byte;
            self.byte_index += 1;
        }
        #[inline(always)]
        fn push_index(&mut self, index: u16) {
            let end_size = 8 - self.accum_size;
            if end_size < self.bit_width {
                // let mask = u16::bitmask_range(0..end_size);
                let overflow = index >> end_size;
                let overflow_size = self.bit_width - end_size;
                let mask = index.get_bitmask(0..end_size) as u8;
                let accum = self.accum | mask << self.accum_size;
                self.accum = 0;
                self.accum_size = 0;
                self.push_byte(accum);
                // wlen += accum.write_to(writer)?;
                if overflow_size >= 8 {
                    let mask = overflow.get_bitmask(0..8) as u8;
                    self.accum = (overflow >> 8) as u8;
                    self.accum_size = overflow_size - 8;
                    self.push_byte(mask);
                } else {
                    self.accum = overflow as u8;
                    self.accum_size = overflow_size;
                }
            } else {// end_size >= bit_width
                // let mask = u16::bitmask_range(0..self.bit_width);
                let mask_start = self.accum_size;
                let mask_end = self.accum_size + self.bit_width;
                self.accum = self.accum.set_bitmask(mask_start..mask_end, index as u8);
                self.accum_size += self.bit_width;
                if self.accum_size == 8 {
                    self.push_byte(self.accum);
                    self.accum = 0;
                    self.accum_size = 0;
                }
            }
        }
    }

    let mut bit_writer = BitWriter {
        accum: 0,
        accum_size: 0,
        bit_width: bit_width as u32,
        bytes: &mut bytes,
        byte_index: 0,
    };
    
    for i in 0..4096 {
        let block_index = id_map[&blocks[i]];
        bit_writer.push_index(block_index);
    }
    // there are still bits in the accumilator, so add the accumilator to the end
    // of the buffer.
    if bit_writer.accum_size > 0 {
        bit_writer.bytes[bit_writer.byte_index] = bit_writer.accum;
    }
    // drop bit_writer to regain access to bytes.
    drop(bit_writer);
    Ok(length + write_bytes(writer, &bytes)?)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    #[test]
    fn write_read_occlusion() -> Result<()> { 
        let occlusions = (0..4096usize).map(|i| Occlusion(i.rem_euclid(64) as u8)).collect::<Box<[_]>>();
        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        write_section_occlusions(&mut buf, &occlusions)?;
        buf.set_position(0);
        let read_occlusions = read_section_occlusions(&mut buf)?;
        assert_eq!(occlusions, read_occlusions);
        Ok(())
    }
}

#[cfg(test)]
mod testing_sandbox {
    use super::*;
    #[test]
    fn sandbox() {
        let v = 0b00001111u32;
        let n = v.next_power_of_two();
        println!("{n:08b} {}", n.trailing_zeros());
    }
}