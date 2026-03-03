use std::fmt;

#[derive(Debug)]
pub enum WindowError {
    InitFailed,
    CreateFailed { width: u32, height: u32, title: String },
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitFailed => write!(f, "Failed to initialize GLFW"),
            Self::CreateFailed { width, height, title } => {
                write!(f, "Failed to create window '{title}' ({width}x{height})")
            }
        }
    }
}

impl std::error::Error for WindowError {}