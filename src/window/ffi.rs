extern crate glfw;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::catch_unwind;
use std::ptr;

use super::events::KgfxEvent;
use super::Window;

fn parse_title(title: *const c_char) -> String {
    if title.is_null() {
        return "[KingoGFX] No window name".to_string();
    }

    unsafe { CStr::from_ptr(title) }
        .to_str()
        .unwrap_or("[KingoGFX] No window name")
        .to_string()
}

#[repr(C)]
pub struct KgfxWindow {
    _private: [u8; 0],
}

fn as_window_mut<'a>(handle: *mut KgfxWindow) -> Option<&'a mut Window> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Window>() })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_create_window(
    title: *const c_char,
    width: u32,
    height: u32,
) -> *mut KgfxWindow {
    let result = catch_unwind(|| {
        let title = parse_title(title);
        Window::new(width, height, title)
            .ok()
            .map(|w| Box::into_raw(Box::new(w)).cast::<KgfxWindow>())
    });

    match result {
        Ok(Some(ptr)) => ptr,
        _ => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_destroy_window(handle: *mut KgfxWindow) {
    if handle.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(handle.cast::<Window>())); }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_focus(handle: *mut KgfxWindow) {
    let Some(w) = as_window_mut(handle) else { return };
    w.focus();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_set_should_close(handle: *mut KgfxWindow, value: bool) {
    let Some(w) = as_window_mut(handle) else { return };
    w.set_should_close(value);
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_poll_events(handle: *mut KgfxWindow) {
    let Some(w) = as_window_mut(handle) else { return };
    let _ = w.poll_events();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_poll_event(
    handle: *mut KgfxWindow,
    out_event: *mut KgfxEvent,
) -> bool {
    if out_event.is_null() {
        return false;
    }

    let Some(w) = as_window_mut(handle) else { return false };
    let Some(ev) = w.poll_event() else { return false };

    unsafe { *out_event = KgfxEvent::from(ev) };
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_swap_buffers(handle: *mut KgfxWindow) {
    let Some(w) = as_window_mut(handle) else { return };
    w.swap_buffers();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_window_should_close(handle: *mut KgfxWindow) -> bool {
    let Some(w) = as_window_mut(handle) else { return true };
    w.should_close()
}