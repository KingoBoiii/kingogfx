extern crate glfw;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use glfw::{Context};

pub struct WindowHandle {
  glfw: glfw::Glfw,
  window: glfw::PWindow,
  events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_create_window(title: *const c_char, width: u32, height: u32) -> *mut WindowHandle {
  let result: Result<Option<*mut WindowHandle>, _> = std::panic::catch_unwind(|| {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let title_str = if title.is_null() {
      "GLFW window"
    } else {
      // Hvis title ikke er valid UTF-8, fallback til default
      unsafe { CStr::from_ptr(title) }
        .to_str()
        .unwrap_or("GLFW window")
    };

    let (mut window, events) = glfw
      .create_window(width, height, title_str, glfw::WindowMode::Windowed)
      ?;

    window.make_current();
    // Valgfrit: slå polling til, hvis du vil håndtere events senere
    window.set_key_polling(true);

    Some(Box::into_raw(Box::new(WindowHandle {
      glfw,
      window,
      events
    })))
  });

  match result {
      Ok(Some(ptr)) => ptr,
      _ => ptr::null_mut()
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_destory_window(handle: *mut WindowHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_poll_events(handle: *mut WindowHandle) {
    if handle.is_null() {
        return;
    }
    let h = unsafe { &mut *handle };
    h.glfw.poll_events();

    // Dræn event queue (valgfrit; men typisk rart at holde den tom)
    for _ in glfw::flush_messages(&h.events) {
        // ... håndtér events hvis du vil eksponere dem via ABI senere
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_swap_buffers(handle: *mut WindowHandle) {
    if handle.is_null() {
        return;
    }
    let h = unsafe { &mut *handle };
    h.window.swap_buffers();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_should_close(handle: *mut WindowHandle) -> bool {
    if handle.is_null() {
        return true;
    }
    let h = unsafe { &mut *handle };
    h.window.should_close()
}