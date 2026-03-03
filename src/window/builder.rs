use crate::window::{Window, backends::glfw::WindowHandle, error::WindowError};

pub struct WindowBuilder {
    title: String,
    width: u32,
    height: u32,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: "KingoGFX".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

impl WindowBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn build(self) -> Result<Window, WindowError> {
        let backend = WindowHandle::create(self.width, self.height, &self.title)?;
        Ok(Window {
            backend: Box::new(backend),
        })
    }
}