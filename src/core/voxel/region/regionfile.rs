use std::{fs::File, io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Take, Write}, path::{Path, PathBuf}};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::{core::error::*, prelude::{write_zeros, Readable, WriteExt, Writeable}};
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
        let parent = path.parent().ok_or(Error::ParentNotFound)?;
        std::fs::create_dir_all(parent)?;
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
            let parent = path.parent().ok_or(Error::ParentNotFound)?;
            std::fs::create_dir_all(parent)?;
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

    pub fn read_value<C: Into<RegionCoord>, T: Readable>(&mut self, coord: C) -> Result<T> {
        #[inline(always)]
        fn read_inner<'a, T: Readable>(mut reader: ZlibDecoder<Take<BufReader<&'a mut File>>>) -> Result<T> {
            T::read_from(&mut reader)
        }
        self.read(coord, read_inner)
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

    pub fn write_value<C: Into<RegionCoord>, T: Writeable>(&mut self, coord: C, value: &T) -> Result<()> {
        self.write(coord, move |writer| {
            value.write_to(writer)?;
            Ok(())
        })
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
fn sandbox() -> Result<()> {
    let mut region = RegionFile::open_or_create("test.dat")?;
    // region.write(RegionCoord::new(1, 3), |writer| {
    //     String::from("The quick brown fox jumps over the lazy dog.").write_to(writer)?;
    //     Ok(())
    // })?;

    let result: String = region.read_value((1, 3))?;

    println!("{result}");

    Ok(())
}