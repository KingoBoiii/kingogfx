use std::ffi::CString;

use kingogfx::window::{kgfx_create_window, kgfx_destory_window, kgfx_window_poll_events, kgfx_window_should_close, kgfx_window_swap_buffers};

fn main() -> () {
  // CString skal leve mindst lige så længe som vinduet bruger pointeren
  let title = CString::new("GLFW window").expect("title contains an interior NUL byte");

  let handle = kgfx_create_window(title.as_ptr(), 800, 600);

  while !kgfx_window_should_close(handle) {
    kgfx_window_poll_events(handle);
    kgfx_window_swap_buffers(handle);
  }

  kgfx_destory_window(handle);
}