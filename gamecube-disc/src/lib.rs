mod fs_table_reader;
mod header_reader;
mod reader;

pub use crate::fs_table_reader::FsTableReader;
pub use crate::header_reader::HeaderReader;
pub use crate::reader::Reader;

/// The size in bytes of a GameCube disc image.
pub const SIZE: usize = 1459978240;
