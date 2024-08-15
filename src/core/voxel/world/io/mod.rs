#![allow(unused)]

use hashbrown::HashMap;
use itertools::Itertools;

use crate::core::voxel::block;
use crate::core::{io::*, voxel::blockstate::BlockState};
use crate::prelude::*;
use crate::core::error::*;

use std::io::{Read, Write};

use super::blockdata::{BlockDataContainer, BlockDataRef};
use super::section::Section;
use super::update::UpdateRef;
use super::VoxelWorld;

pub fn read_section_blocks<R: Read>(reader: &mut R, blocks: &mut Option<Box<[Id]>>, block_count: &mut u16) -> Result<()> {
    let mut state_count = [0u8; 2];
    reader.read_exact(&mut state_count[0..1])?;
    // First byte is null, so the chunk is empty.
    // I can completely avoid even reading this byte if I have
    // flags for which pieces of the section are present.
    *block_count = 0;
    if state_count[0] == 0 {
        *blocks = None;
        return Ok(());
    }
    // If there is a bit at index 7, there is another byte to represent the count.
    let state_count = if state_count[0] & 0b10000000 != 0 {
        reader.read_exact(&mut state_count[1..2])?;
        state_count[0] = state_count[0] & 0b01111111;
        u16::from_be_bytes(state_count) as usize
    } else {
        state_count[0] as usize
    };
    // Only a single block in the entire chunk which fills the whole chunk
    if state_count == 1 {
        let state = BlockState::read_from(reader)?;
        let id = state.register();
        blocks.replace((0..4096).map(|_| id).collect());
        return Ok(());
    }
    // let mut ids = Vec::with_capacity(state_count);
    let ids = (0..state_count).map(|_| {
        let state = BlockState::read_from(reader)?;
        Ok(state.register())
    }).collect::<Result<Box<[Id]>>>()?;
    // This operation would fail if state_count is less than 2, but thankfully
    // it's not going to be.
    let bit_width = state_count.next_power_of_two().trailing_zeros();
    // Multiplied by 512 because 4096 / 8 = 512, and we multiply the bit_width by
    // 4096 then divide by 8, which is the equivalent of multiplying by 512.
    let byte_count = bit_width as usize * 512;
    let bytes = read_bytes(reader, byte_count)?;
    struct BitReader<'a> {
        blocks: &'a mut Box<[Id]>,
        block_count: &'a mut u16,
        block_index: usize,
        accum: u16,
        accum_size: u32,
        bit_width: u32,
        id_table: Box<[Id]>,
    }
    impl<'a> BitReader<'a> {
        
        fn push_id(&mut self, id: Id) {
            if id.is_non_air() {
                *self.block_count += 1;
            }
            self.blocks[self.block_index] = id;
            self.block_index += 1;
        }

        fn push_bits(&mut self, bits: u8, count: u32) {
            if self.block_index == 4096 {
                return;
            }
            let space = self.bit_width - self.accum_size;
            if space >= count {
                let start = self.accum_size;
                let end = start + count;
                self.accum = self.accum.set_bitmask(start..end, bits as u16);
                self.accum_size += count;
                if self.accum_size == self.bit_width {
                    self.push_id(self.id_table[self.accum as usize]);
                    self.accum = 0;
                    self.accum_size = 0;
                }
            } else { // space < count
                self.push_bits(bits, space);
                self.push_bits(bits >> space, count - space);
            }
        }
        
        fn push_byte(&mut self, byte: u8) {
            self.push_bits(byte, 8);
        }
    }
    let mut bitreader = BitReader {
        blocks: blocks.get_or_insert_with(|| (0..4096).map(|_| Id::AIR).collect()),
        block_count,
        block_index: 0,
        accum: 0,
        accum_size: 0,
        bit_width,
        id_table: ids,
    };
    // When this iterator is finished, all Ids should be collected.
    bytes.into_iter().for_each(|byte| {
        bitreader.push_byte(byte);
    });
    assert!(bitreader.accum_size == 0);
    Ok(())
}

pub fn write_section_blocks<W: Write>(writer: &mut W, blocks: &Option<Box<[Id]>>) -> Result<u64> {
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
    // Multiplied by 512 because 4096 / 8 = 512, and we multiply the bit_width by
    // 4096 then divide by 8, which is the equivalent of multiplying by 512.
    let byte_count = bit_width as u32  * 512;
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
        
        fn push_byte(&mut self, byte: u8) {
            self.bytes[self.byte_index] = byte;
            self.byte_index += 1;
        }
        
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
    Ok(length + write_bytes(writer, &bytes)?)
}

/*
0b00111111
0b00001111
0b00000011
0b11111100 (3 bytes, not 4)*/
/// Reads 4096 [Occlusion]s from a reader as 3072 bytes.
// This function will likely want to be inlined
pub fn read_section_occlusions<R: Read>(reader: &mut R, occlusions: &mut Option<Box<[Occlusion]>>, occlusion_count: &mut u16) -> Result<()> {
    let flag = bool::read_from(reader)?;
    *occlusion_count = 0;
    if !flag {
        occlusions.take();
        return Ok(());
    }
    let occlusions = occlusions.get_or_insert_with(|| (0..4096).map(|_| Occlusion::UNOCCLUDED).collect::<Box<[_]>>());
    let mut bytes = [0u8; 6*4096/8];
    reader.read_exact(&mut bytes)?;
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
        if first != 0 {
            *occlusion_count += 1;
        }
        occlusions[occlusion_index+1] = Occlusion(second);
        if second != 0 {
            *occlusion_count += 1;
        }
        occlusions[occlusion_index+2] = Occlusion(third);
        if third != 0 {
            *occlusion_count += 1;
        }
        occlusions[occlusion_index+3] = Occlusion(fourth);
        if fourth != 0 {
            *occlusion_count += 1;
        }
        offset += 3;
        occlusion_index += 4;
    }
    Ok(())
}

// This function will likely want to be inlined.
pub fn write_section_occlusions<W: Write>(writer: &mut W, occlusions: &Option<Box<[Occlusion]>>) -> Result<u64> {
    let Some(occlusions) = occlusions else {
        return false.write_to(writer);
    };
    true.write_to(writer)?;
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
    // Add 1 for the flag that determines if there are occlusions present
    Ok(bytes.len() as u64 + 1)
}

/// For reading both blocklight and skylight.
pub fn read_section_light<R: Read>(reader: &mut R, block_light: &mut Option<Box<[u8]>>, light_count: &mut u16) -> Result<()> {
    let flag = bool::read_from(reader)?;
    *light_count = 0;
    if !flag {
        block_light.take();
        return Ok(());
    }
    // Read 2048 bytes
    let light = block_light.get_or_insert_with(|| (0..2048).map(|_| 0).collect());
    reader.read_exact(light)?;
    *light_count = light.iter().map(|&light| {
        (light & 0x0f != 0) as u16 +
        (light & 0xf0 != 0) as u16
    }).sum();
    Ok(())
}

/// For writing both blocklight and skylight.
pub fn write_section_light<W: Write>(writer: &mut W, block_light: &Option<Box<[u8]>>) -> Result<u64> {
    let Some(light) = block_light else {
        return false.write_to(writer);
    };
    true.write_to(writer)?;
    writer.write_all(light)?;
    Ok(2049)
}

pub fn read_block_data<R: Read>(reader: &mut R, block_data_refs: &mut Option<Box<[BlockDataRef]>>, container: &mut BlockDataContainer, data_count: &mut u16) -> Result<()> {
    container.clear();
    let count = u16::read_from(reader)?;
    *data_count = count;
    if count == 0 {
        block_data_refs.take();
        return Ok(());
    }
    let data = if let Some(data) = block_data_refs {
        data.iter_mut().for_each(|r| *r = BlockDataRef::NULL);
        data
    } else {
        block_data_refs.insert((0..4096).map(|_| BlockDataRef::NULL).collect())
    };
    for _ in 0..count {
        let index = u16::read_from(reader)?;
        let tag = Tag::read_from(reader)?;
        let tag_ref = container.insert(tag);
        data[index as usize] = tag_ref;
    }
    Ok(())
}

pub fn write_block_data<W: Write>(writer: &mut W, block_data_refs: &Option<Box<[BlockDataRef]>>, container: &BlockDataContainer, data_count: u16) -> Result<u64> {
    let Some(data) = block_data_refs else {
        return 0u16.write_to(writer);
    };
    // let's assume there might be 256 blocks that have data
    let mut data_ids = Vec::with_capacity(data_count as usize);
    for (i, &id) in data.iter().enumerate() {
        if !id.null() {
            data_ids.push((i as u16, id));
        }
    }
    let mut length = (data_ids.len() as u16).write_to(writer)?;
    for (index, id) in data_ids {
        let tag = container.get(id).expect("Data container corruption");
        length += index.write_to(writer)?;
        length += tag.write_to(writer)?;
    }
    Ok(length)
}

/// The enabled callback will receive the index (within the section) to enable a block.
/// It's up to the caller to use that index to enable a block.
pub fn read_enabled<R: Read, F: FnMut(u16)>(reader: &mut R, mut enable: F, enabled_count: &mut u16) -> Result<()> {
    let count = u16::read_from(reader)?;
    *enabled_count = count;
    if count == 0 {
        return Ok(());
    }
    for i in 0..count {
        let index = u16::read_from(reader)?;
        enable(index);
    }
    Ok(())
}

pub fn write_enabled<W: Write>(writer: &mut W, update_refs: &Option<Box<[UpdateRef]>>) -> Result<u64> {
    let Some(refs) = update_refs else {
        return 0u16.write_to(writer);
    };
    // get the enabled count
    let mut updates = Vec::with_capacity(256);
    for (i, &uref) in refs.iter().enumerate() {
        if !uref.null() {
            updates.push(i as u16);
        }
    }
    let mut len = (updates.len() as u16).write_to(writer)?;
    len += updates.len() as u64 * 2;
    for update in updates.into_iter() {
        update.write_to(writer)?;
    }
    Ok(len)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::buffer;

    use crate::{blockstate, core::util::traits::StrToOwned};

    use super::*;
    #[test]
    fn write_read_occlusion() -> Result<()> { 
        {    
            let occlusions = Some((0..4096usize).map(|i| Occlusion(i.rem_euclid(64) as u8)).collect::<Box<[_]>>());
            let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            write_section_occlusions(&mut buf, &occlusions)?;
            buf.set_position(0);
            let mut read_occlusions = None;
            let mut count = 10;
            read_section_occlusions(&mut buf, &mut read_occlusions, &mut count)?;
            assert_ne!(count, 10);
            assert_eq!(occlusions, read_occlusions);
        }
        {    
            let occlusions = None;
            let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            write_section_occlusions(&mut buf, &occlusions)?;
            buf.set_position(0);
            let mut read_occlusions = Some((0..4096usize).map(|i| Occlusion(i.rem_euclid(64) as u8)).collect::<Box<[_]>>());
            let mut count = 10;
            read_section_occlusions(&mut buf, &mut read_occlusions, &mut count)?;
            assert_ne!(count, 10);
            assert_eq!(occlusions, read_occlusions);
        }
        {    
            let occlusions = Some((0..4096usize).map(|i| Occlusion(i.rem_euclid(64) as u8 | 1)).collect::<Box<[_]>>());
            let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            write_section_occlusions(&mut buf, &occlusions)?;
            buf.set_position(0);
            let mut read_occlusions = Some((0..4096usize).map(|i| Occlusion::OCCLUDED).collect::<Box<[_]>>());
            let mut count = 10;
            read_section_occlusions(&mut buf, &mut read_occlusions, &mut count)?;
            assert_eq!(count, 4096);
            assert_eq!(occlusions, read_occlusions);
        }
        Ok(())
    }

    #[test]
    fn write_read_blocks() -> Result<()> {
        struct TestBlock {
            name: String,
        }
        impl TestBlock {
            fn new<S: StrToOwned>(name: S) -> Self {
                Self {
                    name: name.str_to_owned()
                }
            }
        }

        impl Block for TestBlock {
            fn name(&self) -> &str {
                &self.name
            }
        
            fn default_state(&self) -> BlockState {
                BlockState::new(&self.name, [])
            }
        }
        blocks::register_block(TestBlock::new("testblock".to_owned()));
        // register blocks
        {
            let mut blocks: Option<Box<[Id]>> = Some((0i64..4096).map(|i| blockstate!(testblock, i=i.rem_euclid(300)).register()).collect());
            
            let mut buffy = Cursor::new(Vec::<u8>::new());
            write_section_blocks(&mut buffy, &blocks)?;
            let mut read_blocks: Option<Box<[Id]>> = Some((0..4096).map(|i| Id::AIR).collect());
            buffy.set_position(0);
            let mut count = 0u16;
            read_section_blocks(&mut buffy, &mut read_blocks, &mut count)?;
            assert_eq!(count, 4096);
            assert_eq!(blocks, read_blocks);
        }
        {
            let mut blocks: Option<Box<[Id]>> = None;
            let mut buffy = Cursor::new(Vec::<u8>::new());
            write_section_blocks(&mut buffy, &blocks)?;
            let mut read_blocks: Option<Box<[Id]>> = Some((0..4096).map(|i| Id::AIR).collect());
            buffy.set_position(0);
            let mut count = 0u16;
            read_section_blocks(&mut buffy, &mut read_blocks, &mut count)?;
            assert_eq!(count, 0);
            assert_eq!(blocks, read_blocks);
        }
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