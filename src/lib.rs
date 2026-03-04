use crate::window::{Input, KeyCode, KeyEvent};

pub mod window;
pub mod graphics;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_init() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shutdown() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed(event: KeyEvent, key_code: KeyCode) -> bool {
    Input::is_key_released(event, key_code)
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_released(event: KeyEvent, key_code: KeyCode) -> bool {
    Input::is_key_released(event, key_code)
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed_i32(event: KeyEvent, key_code: i32) -> bool {
    matches!(KeyCode::from_i32(key_code), Some(k) if Input::is_key_released(event, k))
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_released_i32(event: KeyEvent, key_code: i32) -> bool {
    matches!(KeyCode::from_i32(key_code), Some(k) if Input::is_key_released(event, k))
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
