use std::fmt::{self, Display, Formatter};

pub struct FormatSmallI16(pub i16);

impl Display for FormatSmallI16 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.0 > 9 {
            write!(f, "{:#x}", self.0)
        } else if self.0 >= -9 {
            write!(f, "{}", self.0)
        } else {
            write!(f, "-{:#x}", -(self.0 as i32))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FormatSmallI16;

    #[test]
    fn format_small_i16() {
        assert_eq!(FormatSmallI16(0).to_string(), "0");
        assert_eq!(FormatSmallI16(-9).to_string(), "-9");
        assert_eq!(FormatSmallI16(9).to_string(), "9");
        assert_eq!(FormatSmallI16(-10).to_string(), "-0xa");
        assert_eq!(FormatSmallI16(10).to_string(), "0xa");
        assert_eq!(FormatSmallI16(32767).to_string(), "0x7fff");
        assert_eq!(FormatSmallI16(-32768).to_string(), "-0x8000");
    }
}
