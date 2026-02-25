use crate::window::{Key, KeyAction, KeyEvent};

pub mod window;
pub mod graphics;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_init() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shutdown() -> () {
}

pub fn is_key_pressed(event: KeyEvent, key: Key) -> bool {
    event.key == key && event.action == KeyAction::Press
}

pub fn is_key_released(event: KeyEvent, key: Key) -> bool {
    event.key == key && event.action == KeyAction::Release
}

pub fn kgfx_is_key_pressed(event: KeyEvent, key: Key) -> bool {
    is_key_pressed(event, key)
}

pub fn kgfx_is_key_released(event: KeyEvent, key: Key) -> bool {
    is_key_released(event, key)
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed_i32(event: KeyEvent, key: i32) -> bool {
    matches!(Key::from_i32(key), Some(k) if is_key_pressed(event, k))
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_released_i32(event: KeyEvent, key: i32) -> bool {
    matches!(Key::from_i32(key), Some(k) if is_key_released(event, k))
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
