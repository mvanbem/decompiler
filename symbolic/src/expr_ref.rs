#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExprRef(pub(crate) usize);

impl ExprRef {
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }

    pub fn to_raw(self) -> usize {
        self.0
    }
}
