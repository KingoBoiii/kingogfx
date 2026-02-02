extern crate glfw;

use std::collections::VecDeque;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use glfw::Context;

use super::events::{KgfxEvent, map_event};
use super::handle::WindowHandle;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_create_window(title: *const c_char, width: u32, height: u32) -> *mut WindowHandle {
  let result: Result<Option<*mut WindowHandle>, _> = std::panic::catch_unwind(|| {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let title_str = if title.is_null() {
      "GLFW window"
    } else {
      unsafe { CStr::from_ptr(title) }
        .to_str()
        .unwrap_or("GLFW window")
    };

    let (mut window, events) = glfw
      .create_window(width, height, title_str, glfw::WindowMode::Windowed)
      ?;

    window.make_current();
    window.set_key_polling(true);

    Some(Box::into_raw(Box::new(WindowHandle {
      glfw,
      window,
      events,
      event_queue: VecDeque::new(),
    })))
  });

  match result {
      Ok(Some(ptr)) => ptr,
      _ => ptr::null_mut()
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_destroy_window(handle: *mut WindowHandle) {
  if handle.is_null() {
    return;
  }
  unsafe { drop(Box::from_raw(handle)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_set_should_close(handle: *mut WindowHandle, value: bool) {
  if handle.is_null() {
    return;
  }
  let h = unsafe { &mut *handle };
  h.window.set_should_close(value);
}

/// Poller OS-events ind i GLFW (returnerer ikke events)
#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_poll_events(handle: *mut WindowHandle) {
  if handle.is_null() {
    return;
  }
  let h = unsafe { &mut *handle };
  h.glfw.poll_events();
}

/// “Tøm én event”-API (FFI-venlig):
/// Returnerer `true` hvis der var et event, ellers `false`.
/// Intern logik: poll + flush -> queue -> pop 1.
#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_poll_event(handle: *mut WindowHandle, out_event: *mut KgfxEvent) -> bool {
  if handle.is_null() || out_event.is_null() {
    return false;
  }
  let h = unsafe { &mut *handle };

  // Hvis køen er tom, så hent nye events fra GLFW
  if h.event_queue.is_empty() {
    h.glfw.poll_events();
    for (_, event) in glfw::flush_messages(&h.events) {
      h.event_queue.push_back(event);
    }
  }

  let Some(ev) = h.event_queue.pop_front() else {
    return false;
  };

  unsafe { *out_event = map_event(ev); }
  true
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