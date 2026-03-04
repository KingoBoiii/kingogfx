use std::ffi::c_void;

use super::WindowEvent;

pub(crate) trait WindowBackend {
    fn get_proc_address(&mut self, procname: &str) -> *const c_void;
    fn make_current(&mut self);
    fn poll_event(&mut self) -> Option<WindowEvent>;
    fn poll_events(&mut self) -> Vec<WindowEvent>;
    fn swap_buffers(&mut self);
    fn focus(&mut self);
    fn should_close(&self) -> bool;
    fn set_should_close(&mut self, value: bool);

    fn framebuffer_size(&self) -> (i32, i32);
    fn glfw_window_ptr(&mut self) -> *mut glfw_sys::GLFWwindow;
}