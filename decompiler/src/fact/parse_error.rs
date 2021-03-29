use std::any::Any;

use powerpc::ParseError;

use crate::fact::Fact;

#[derive(Debug)]
pub struct ParseErrorFact(ParseError);

impl ParseErrorFact {
    pub fn new(err: ParseError) -> Self {
        Self(err)
    }

    pub fn parse_error(&self) -> &ParseError {
        &self.0
    }
}

impl Fact for ParseErrorFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
