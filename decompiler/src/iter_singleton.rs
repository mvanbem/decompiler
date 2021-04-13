pub trait IteratorExt: Iterator {
    /// Consumes the iterator, returning an element if there is precisely one, or `None` otherwise.
    fn singleton(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let element = self.next()?;
        match self.next() {
            Some(_) => None,
            None => Some(element),
        }
    }
}

impl<T: Iterator> IteratorExt for T {}
