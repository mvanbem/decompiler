use std::iter::{ExactSizeIterator, FusedIterator};

use crate::{Reader, Section, SECTION_COUNT};

pub struct SectionsIter<'data> {
    pub(crate) reader: Reader<'data>,
    pub(crate) index: usize,
}

impl<'data> Iterator for SectionsIter<'data> {
    type Item = Section;

    fn next(&mut self) -> Option<Section> {
        if self.index < SECTION_COUNT {
            let result = Some(self.reader.section(self.index));
            self.index += 1;
            result
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = SECTION_COUNT - self.index;
        (len, Some(len))
    }
}

impl<'data> ExactSizeIterator for SectionsIter<'data> {}

impl<'data> FusedIterator for SectionsIter<'data> {}
