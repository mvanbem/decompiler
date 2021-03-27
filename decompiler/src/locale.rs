use std::fmt::{self, Display, Formatter};

use lazy_static::lazy_static;
use num_format::{SystemLocale, ToFormattedString, WriteFormatted};

lazy_static! {
    static ref DEFAULT_SYSTEM_LOCALE: SystemLocale = SystemLocale::default().unwrap();
}

pub struct LocaleFormat<'a, N: ToFormattedString>(pub &'a N);

impl<'a, N: ToFormattedString> Display for LocaleFormat<'a, N> {
    fn fmt(&self, mut f: &mut Formatter) -> fmt::Result {
        match f.write_formatted(self.0, &*DEFAULT_SYSTEM_LOCALE) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
