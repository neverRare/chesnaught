use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidByte;

impl Display for InvalidByte {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid byte")?;
        Ok(())
    }
}
impl Error for InvalidByte {}
