use std::ffi::CString;

use kingogfx::window::{
  KgfxEvent, KgfxEventKind,
  kgfx_create_window, kgfx_destroy_window, kgfx_window_poll_event,
  kgfx_window_set_should_close, kgfx_window_should_close, kgfx_window_swap_buffers,
};
use kingogfx::graphics::{
  kgfx_graphics_load_gl, 
  kgfx_graphics_clear, kgfx_graphics_clear_color
};
use kingogfx::{kgfx_is_key_pressed};

fn main() {
  let title = CString::new("GLFW window").expect("title contains an interior NUL byte");
  let handle = kgfx_create_window(title.as_ptr(), 800, 600);

  kgfx_graphics_load_gl(handle);

  let mut event = KgfxEvent::default();

  while !kgfx_window_should_close(handle) {
    kgfx_graphics_clear();
    kgfx_graphics_clear_color(0.1, 0.2, 0.3, 1.0);

    while kgfx_window_poll_event(handle, &mut event) {
      match event.kind {
        KgfxEventKind::Key => {
          if let Some(k) = event.as_key() {
            // key 256 = escape key
            if kgfx_is_key_pressed(k, 256) {
              kgfx_window_set_should_close(handle, true);
            }
          }
        }

        _ => {}
      }
    }

    kgfx_window_swap_buffers(handle);
  }

  kgfx_destroy_window(handle);
}