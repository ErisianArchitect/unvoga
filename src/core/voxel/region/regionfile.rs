use std::{fs::File, io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Take, Write}, path::{Path, PathBuf}};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::{core::error::*, prelude::{write_zeros, Readable, Writeable}};
use super::{header::RegionHeader, regioncoord::RegionCoord, sectormanager::SectorManager, sectoroffset::BlockSize};

pub struct RegionFile {
    sector_manager: SectorManager,
    /// Used for both reading and writing. The file is kept locked while the region is open.
    io: File,
    write_buffer: Cursor<Vec<u8>>,
    header: RegionHeader,
    path: PathBuf,
}

impl RegionFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file_handle = File::options()
            .read(true)
            .write(true)
            .open(path.as_ref())?;
        let file_size = file_handle.seek(SeekFrom::End(0))?;
        // The file is too small to contain the header.
        if file_size < 4096 + 8192 {
            return Err(Error::NoHead);
        }
        file_handle.seek(SeekFrom::Start(0))?;
        let header = {
            let mut temp_reader = BufReader::new((&mut file_handle).take(4096*3));
            RegionHeader::read_from(&mut temp_reader)?
        };
        let sector_manager = SectorManager::from_sector_table(&header.offsets);
        Ok(Self {
            io: file_handle,
            header,
            sector_manager,
            write_buffer: Cursor::new(Vec::with_capacity(4096*2)),
            path: path.as_ref().to_owned(),
        })
    }

    /// Returns error if the file already exists.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut io = File::options()
            .read(true).write(true)
            .create_new(true)
            .open(path)?;
        write_zeros(&mut io, 4096*3)?;
        Ok(Self {
            io,
            write_buffer: Cursor::new(Vec::with_capacity(4096*2)),
            header: RegionHeader::new(),
            sector_manager: SectorManager::new(),
            path: path.to_owned()
        })
    }

    /// Returns error if the path exists but isn't a file.
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            Self::create(path)
        } else if path.is_file() {
            Self::open(path)
        } else {
            Err(Error::NotAFile)
        }
    }

    pub fn read<'a, C: Into<RegionCoord>, R, F: FnMut(ZlibDecoder<Take<BufReader<&'a mut File>>>) -> Result<R>>(&'a mut self, coord: C, mut read: F) -> Result<R> {
        let coord: RegionCoord = coord.into();
        let sector = self.header.offsets[coord];
        if sector.is_empty() {
            return Err(Error::ChunkNotFound);
        }
        let mut reader = BufReader::new(&mut self.io);
        reader.seek(SeekFrom::Start(sector.file_offset()))?;
        let length = u32::read_from(&mut reader)?;
        let decoder = ZlibDecoder::new(reader.take(length as u64));
        read(decoder)
    }

    pub fn write<C: Into<RegionCoord>, F: FnMut(&mut ZlibEncoder<&mut Cursor<Vec<u8>>>) -> Result<()>>(&mut self, coord: C, mut write: F) -> Result<()> {
        let coord: RegionCoord = coord.into();
        self.write_buffer.get_mut().clear();
        self.write_buffer.write_all(&[0u8; 4])?;
        let mut encoder = ZlibEncoder::new(&mut self.write_buffer, Compression::best());
        write(&mut encoder)?;
        encoder.finish()?;
        let length = self.write_buffer.get_ref().len() as u64;
        let padded_size = padded_size(length);
        if padded_size > BlockSize::MAX_BLOCK_COUNT as u64 * 4096 {
            return Err(Error::ChunkTooLarge);
        }
        let block_size = (padded_size / 4096) as u16;
        let required_size = BlockSize::required(block_size);
        self.write_buffer.set_position(0);
        (length as u32).write_to(&mut self.write_buffer)?;
        let old_sector = self.header.offsets[coord];
        let new_sector = self.sector_manager.reallocate_err(old_sector, required_size)?;
        self.header.offsets[coord] = new_sector;
        let mut writer = BufWriter::new(&mut self.io);
        writer.seek(SeekFrom::Start(new_sector.block_offset() as u64 * 4096))?;
        writer.write_all(self.write_buffer.get_ref().as_slice())?;
        write_zeros(&mut writer, pad_size(length as u64))?;
        writer.seek(SeekFrom::Start(coord.sector_offset()))?;
        new_sector.write_to(&mut writer)?;
        writer.flush()?;
        Ok(())
    }
}

#[inline(always)]
fn pad_size(length: u64) -> u64 {
    4096 - (length & 4095) & 4095
}

#[inline(always)]
fn padded_size(length: u64) -> u64 {
    length + pad_size(length)
}

#[test]
fn sandbox() {
    assert_eq!(pad_size(5), 4096)
}