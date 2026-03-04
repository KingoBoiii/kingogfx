pub mod builder;
pub mod error;
pub mod events;
pub mod ffi;
pub mod input;
pub(crate) mod backend;
pub(crate) mod backends;

use std::{ffi::c_void};

use backend::WindowBackend;

pub use events::*;
pub use input::*;

use crate::{window::{builder::WindowBuilder, error::WindowError}};

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

    pub(crate) fn framebuffer_size(&self) -> (i32, i32) {
        self.backend.framebuffer_size()
    }

    pub(crate) fn glfw_window_ptr(&mut self) -> *mut glfw_sys::GLFWwindow {
        self.backend.glfw_window_ptr()
    }
}

pub type KgfxKeyCode = KeyCode;
pub type KgfxKeyModifiers = KeyModifiers;
pub type KgfxKeyEvent = KeyEvent;