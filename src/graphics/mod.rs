pub mod ffi;

pub use ffi::*;

use crate::window::handle::WindowHandle;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_load_gl(window_handle: *mut WindowHandle) -> () {
  let window_handle = unsafe { window_handle.as_mut() };
  let Some(window_handle) = window_handle else {
    return; // evt. log/return error-kode via FFI, hvis du vil
  };

  gl::load_with(|symbol| {
    match window_handle.window.get_proc_address(symbol) {
      Some(proc_addr) => proc_addr as *const _,
      None => std::ptr::null(),
    }
  });
}