/// The size in bytes of a GameCube disc header.
pub const HEADER_SIZE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct HeaderReader<'data> {
    data: &'data [u8],
}

impl<'data> HeaderReader<'data> {
    /// # Panics
    ///
    /// Panics if `data.len()` is less than [`HEADER_SIZE`].
    pub fn new(data: &'data [u8]) -> HeaderReader<'data> {
        HeaderReader {
            data: &data[..HEADER_SIZE],
        }
    }

    pub fn game_code(&self) -> String {
        self.data[0..4]
            .iter()
            .copied()
            .map(|c| (c as char).escape_default())
            .flatten()
            .collect()
    }

    pub fn maker_code(&self) -> String {
        self.data[4..6]
            .iter()
            .copied()
            .map(|c| (c as char).escape_default())
            .flatten()
            .collect()
    }

    pub fn disc_id(&self) -> u8 {
        self.data[6]
    }

    pub fn version(&self) -> u8 {
        self.data[7]
    }
}

#[cfg(test)]
pub mod tests {
    use super::HeaderReader;

    #[test]
    fn test() {
        const DATA: &'static [u8] = &[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x01, 0x02];
        let header = HeaderReader::new(DATA);
        assert_eq!(header.game_code(), "ABCD");
        assert_eq!(header.maker_code(), "EF");
        assert_eq!(header.disc_id(), 1);
        assert_eq!(header.version(), 2);
    }
}
