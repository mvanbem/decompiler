use byteorder::{BigEndian, ReadBytesExt};

use crate::{Section, SectionsIter, SECTION_COUNT};

const ENTRY_POINT_OFFSET: usize = 0xe0;
const SECTION_OFFSET_TABLE_OFFSET: usize = 0;
const SECTION_LOAD_ADDRESS_TABLE_OFFSET: usize = 0x48;
const SECTION_SIZE_TABLE_OFFSET: usize = 0x90;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Reader<'data> {
    data: &'data [u8],
}

impl<'data> Reader<'data> {
    pub fn new(data: &'data [u8]) -> Reader<'data> {
        let mut reader = Reader { data };

        // Bound the data slice.
        let farthest_end = reader
            .iter_sections()
            .map(|section| section.offset + section.size)
            .max()
            .unwrap() as usize;
        reader.data = &data[..farthest_end];
        reader
    }

    pub fn section(self, index: usize) -> Section {
        if index >= SECTION_COUNT {
            panic!("index out of range: {}", index);
        }
        Section {
            offset: (&self.data[4 * index + SECTION_OFFSET_TABLE_OFFSET..])
                .read_u32::<BigEndian>()
                .unwrap(),
            load_address: (&self.data[4 * index + SECTION_LOAD_ADDRESS_TABLE_OFFSET..])
                .read_u32::<BigEndian>()
                .unwrap(),
            size: (&self.data[4 * index + SECTION_SIZE_TABLE_OFFSET..])
                .read_u32::<BigEndian>()
                .unwrap(),
        }
    }

    pub fn iter_sections(self) -> SectionsIter<'data> {
        SectionsIter {
            reader: self,
            index: 0,
        }
    }

    pub fn entry_point(self) -> u32 {
        (&self.data[ENTRY_POINT_OFFSET..])
            .read_u32::<BigEndian>()
            .unwrap()
    }

    pub fn read(self, address: u32) -> u32 {
        for section in self.iter_sections() {
            if let Some(offset) = section.offset_for_dword_address(address) {
                return (&self.data[offset as usize..])
                    .read_u32::<BigEndian>()
                    .unwrap();
            }
        }
        panic!("address not mapped: {:08x}", address);
    }
}
