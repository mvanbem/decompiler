mod reader;
mod section;
mod sections_iter;

pub use crate::reader::Reader;
pub use crate::section::Section;
pub use crate::sections_iter::SectionsIter;

pub const SECTION_COUNT: usize = 18;
