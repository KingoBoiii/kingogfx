use std::fmt;

#[derive(Debug)]
pub struct GraphicsError(pub String);

impl From<String> for GraphicsError {
    fn from(err: String) -> Self {
        GraphicsError(err)
    }
}

impl fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GraphicsError {}