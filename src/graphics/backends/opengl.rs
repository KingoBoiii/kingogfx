use glfw::Context;

use crate::window::handle::WindowHandle;
use std::ffi::c_void;

pub struct OpenGLBackend;

impl OpenGLBackend {
    pub fn new(window: &mut WindowHandle) -> Option<Self> {
        // GLFW/OpenGL: context skal være current før du loader pointers
        window.window.make_current();

        gl::load_with(|symbol| match window.window.get_proc_address(symbol) {
            Some(proc_addr) => proc_addr as *const c_void,
            None => std::ptr::null(),
        });

        Some(Self)
    }

    pub fn clear(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }
}