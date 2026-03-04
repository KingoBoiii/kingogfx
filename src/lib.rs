use crate::window::{Input, KeyCode, KgfxEvent};

pub mod window;
pub mod graphics;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_init() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shutdown() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed(event: KgfxEvent, key_code: KeyCode) -> bool {
    matches!(event.as_key(), Some(k) if Input::is_key_pressed(k, key_code))
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_released(event: KgfxEvent, key_code: KeyCode) -> bool {
    matches!(event.as_key(), Some(k) if Input::is_key_released(k, key_code))
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed_i32(event: KgfxEvent, key_code: i32) -> bool {
    let Some(key_code) = KeyCode::from_i32(key_code) else {
        return false;
    };
    matches!(event.as_key(), Some(k) if Input::is_key_pressed(k, key_code))
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_released_i32(event: KgfxEvent, key_code: i32) -> bool {
    let Some(key_code) = KeyCode::from_i32(key_code) else {
        return false;
    };
    matches!(event.as_key(), Some(k) if Input::is_key_released(k, key_code))
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
