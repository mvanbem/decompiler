#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VariableRef(pub(crate) usize);

impl VariableRef {
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }

    pub fn to_raw(self) -> usize {
        self.0
    }
}
