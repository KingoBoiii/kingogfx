use crate::window::{Window, backends::glfw::WindowHandle, error::WindowError};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WindowClientApi {
    OpenGl,
    NoApi,
}

pub struct WindowBuilder {
    title: String,
    width: u32,
    height: u32,
    client_api: WindowClientApi,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: "KingoGFX".to_string(),
            width: 1280,
            height: 720,
            client_api: WindowClientApi::OpenGl,
        }
    }
}

impl WindowBuilder {
    pub fn client_api(mut self, client_api: WindowClientApi) -> Self {
        self.client_api = client_api;
        self
    }

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
        let backend = WindowHandle::create(self.width, self.height, &self.title, self.client_api)?;
        Ok(Window {
            backend: Box::new(backend),
        })
    }
}