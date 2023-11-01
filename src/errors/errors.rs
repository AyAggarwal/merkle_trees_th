use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    BelowZero,
    Invalid,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValidationError::BelowZero => write!(f, "Input can only accept positive values"),
            ValidationError::Invalid => write!(f, "Input is invalid"),
        }
    }
}
