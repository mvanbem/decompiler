#[derive(Clone, Copy, Debug)]
pub struct Section {
    pub offset: u32,
    pub load_address: u32,
    pub size: u32,
}

impl Section {
    pub fn offset_for_dword_address(self, address: u32) -> Option<u32> {
        let offset_within_section = address.checked_sub(self.load_address)?;
        let final_offset_within_section = offset_within_section.checked_add(3)?;

        if final_offset_within_section < self.size {
            Some(self.offset + offset_within_section)
        } else {
            None
        }
    }
}
