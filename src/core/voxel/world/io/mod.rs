
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

pub fn write_section_blocks<W: Write>(writer: &mut W, blocks: &[Id]) -> Result<u64> {
    assert_eq!(blocks.len(), 4096, "Expected a length of 4096, got a length of {}", blocks.len());
    // Map present blocks to new ids.
    let mut id_map = HashMap::<Id, u16>::new();
    let mut id_counter = 0u16;
    let mut new_ids = [0u16; 4096];
    for i in 0..4096 {
        let entry = id_map.entry(blocks[i]).or_insert(id_counter);
        if *entry == id_counter {
            id_counter += 1;
        }
        new_ids[i] = *entry;
    }
    // There should be a flag for if there is only one block type.
    // I guess that's the length of the block table.
    // But what if that block is air? We don't even need a block table.
    if id_counter == 1 && blocks[0] == Id::AIR {
        return 0u8.write_to(writer);
    } else {
        todo!()
    }
    todo!()
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
        let v = 0x03020100u32;
        let b = v.to_be_bytes();
        println!("{b:?}");
    }
}