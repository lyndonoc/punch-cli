use std::fmt::{self, Formatter};

#[derive(Clone, Debug)]
pub struct SimpleError {
    pub message: String,
}

pub type SimpleMessageResult<T> = std::result::Result<T, SimpleError>;

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message.clone())
    }
}
