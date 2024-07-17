use std::{fs::File, io::{Cursor, Seek, SeekFrom}, path::{Path, PathBuf}};

use crate::core::error::*;
use super::{header::RegionHeader, sectormanager::SectorManager};

pub struct RegionFile {
    sector_manage: SectorManager,
    /// Used for both reading and writing. The file is kept locked while the region is open.
    io: File,
    write_buffer: Cursor<Vec<u8>>,
    header: RegionHeader,
    path: PathBuf,
}

impl RegionFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file_handle = File::options().read(true).write(true).open(path)?;
        let file_size = file_handle.seek(SeekFrom::End(0))?;
        // The file is too small to contain the header.
        if file_size < 4096 + 8192 {
            
        }
    }

    pub fn create<P: AsRef<Path>>(path: P, create_new: bool) -> Result<Self> {
        
    }

    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        todo!()
    }
}