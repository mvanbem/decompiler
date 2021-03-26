use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

use crate::header_reader::HeaderReader;
use crate::{FsTableReader, SIZE};

const MAIN_EXECUTABLE_OFFSET: usize = 0x420;
const FILESYSTEM_TABLE_OFFSET_OFFSET: usize = 0x424;
const FILESYSTEM_TABLE_LENGTH_OFFSET: usize = 0x428;

#[derive(Clone, Copy, Debug)]
pub struct Reader<'data> {
    data: &'data [u8],
}

impl<'data> Reader<'data> {
    /// # Panics
    ///
    /// Panics if `data.len()` is less than [`SIZE`].
    pub fn new(data: &'data [u8]) -> Reader<'data> {
        Reader {
            data: &data[..SIZE],
        }
    }

    pub fn header(&self) -> HeaderReader<'data> {
        HeaderReader::new(self.data)
    }

    pub fn main_executable(&self) -> dol::Reader<'data> {
        let offset = (&self.data[MAIN_EXECUTABLE_OFFSET..])
            .read_u32::<BigEndian>()
            .unwrap() as usize;
        dol::Reader::new(&self.data[offset..])
    }

    pub fn fs_table(&self) -> FsTableReader<'data> {
        let offset = (&self.data[FILESYSTEM_TABLE_OFFSET_OFFSET..])
            .read_u32::<BigEndian>()
            .unwrap() as usize;
        let len = (&self.data[FILESYSTEM_TABLE_LENGTH_OFFSET..])
            .read_u32::<BigEndian>()
            .unwrap() as usize;
        FsTableReader::new(&self.data[offset..(offset + len)])
    }

    pub fn find_file(&self, _path: &Path) -> Option<&[u8]> {
        // TODO
        None
    }
}
