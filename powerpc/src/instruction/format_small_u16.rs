use std::fmt::{self, Display, Formatter};

pub struct FormatSmallU16(pub u16);

impl Display for FormatSmallU16 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.0 > 9 {
            write!(f, "{:#x}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FormatSmallU16;

    #[test]
    fn format_small_u16() {
        assert_eq!(FormatSmallU16(0).to_string(), "0");
        assert_eq!(FormatSmallU16(9).to_string(), "9");
        assert_eq!(FormatSmallU16(10).to_string(), "0xa");
        assert_eq!(FormatSmallU16(32767).to_string(), "0x7fff");
        assert_eq!(FormatSmallU16(32768).to_string(), "0x8000");
        assert_eq!(FormatSmallU16(65535).to_string(), "0xffff");
    }
}
