use std::ffi::CString;

use kingogfx::kgfx_is_key_pressed;
use kingogfx::window::KgfxKey;
use kingogfx::window::ffi::kgfx_window_focus;
use kingogfx::window::{
    ffi::{
        kgfx_create_window, kgfx_destroy_window, kgfx_window_poll_event, kgfx_window_set_should_close,
        kgfx_window_should_close, kgfx_window_swap_buffers,
    },
    KgfxEvent, KgfxEventKind,
};

fn main() {
    let title = CString::new("KingoGFX - Window Example (FFI)").expect("title contains interior NUL");
    let handle = kgfx_create_window(title.as_ptr(), 1280, 720);

    if handle.is_null() {
        eprintln!("kgfx_create_window failed");
        return;
    }

    kgfx_window_focus(handle);

    let mut event = KgfxEvent::default();

    while !kgfx_window_should_close(handle) {
        while kgfx_window_poll_event(handle, &mut event) {
            if let KgfxEventKind::KeyEvent = event.kind {
                if let Some(k) = event.as_key() {
                    println!(
                        "Key event -> key: {:?}, action: {:?}, mods: {:?}",
                        k.key, k.action, k.modifiers
                    );

                    if kgfx_is_key_pressed(k, KgfxKey::Escape) {
                        kgfx_window_set_should_close(handle, true);
                    }
                }
            }
        }

        kgfx_window_swap_buffers(handle);
    }

    kgfx_destroy_window(handle);
}