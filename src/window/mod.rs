pub mod events;
pub mod ffi;
pub mod input;
pub(crate) mod backend;
pub(crate) mod backends;

use std::{ffi::c_void, fmt};

use backend::WindowBackend;
use backends::glfw::WindowHandle;

pub use events::*;
pub use input::*;

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

pub struct Window {
    backend: Box<dyn WindowBackend>,
}

impl Window {
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }

    pub fn new(width: u32, height: u32, title: impl Into<String>) -> Result<Self, WindowError> {
        Window::builder().size(width, height).title(title).build()
    }

    pub fn get_proc_address(&mut self, procname: &str) -> *const c_void {
        self.backend.get_proc_address(procname)
    }

    pub fn make_current(&mut self) {
        self.backend.make_current();
    }

    pub fn poll_event(&mut self) -> Option<WindowEvent> {
        self.backend.poll_event()
    }

    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        self.backend.poll_events()
    }

    pub fn swap_buffers(&mut self) {
        self.backend.swap_buffers();
    }

    pub fn focus(&mut self) {
        self.backend.focus();
    }

    pub fn should_close(&self) -> bool {
        self.backend.should_close()
    }

    pub fn set_should_close(&mut self, value: bool) {
        self.backend.set_should_close(value);
    }
}

pub type KgfxKey = Key;
pub type KgfxKeyModifiers = KeyModifiers;
pub type KgfxKeyEvent = KeyEvent;