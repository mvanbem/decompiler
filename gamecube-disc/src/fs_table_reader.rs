use byteorder::{BigEndian, ReadBytesExt};

const ROOT_ENTRY_COUNT_OFFSET: usize = 0x8;

const STRING_TABLE_ENTRY_SIZE: usize = 0xc;

#[derive(Clone, Copy, Debug)]
pub struct FsTableReader<'data> {
    data: &'data [u8],
}

impl<'data> FsTableReader<'data> {
    pub fn new(data: &'data [u8]) -> FsTableReader<'data> {
        FsTableReader { data }
    }

    pub fn root_entry_count(&self) -> u32 {
        (&self.data[ROOT_ENTRY_COUNT_OFFSET..])
            .read_u32::<BigEndian>()
            .unwrap()
    }

    pub fn string_table(&self) -> &'data [u8] {
        &self.data[(self.root_entry_count() as usize * STRING_TABLE_ENTRY_SIZE)..]
    }
}
