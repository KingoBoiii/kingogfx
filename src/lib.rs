use crate::window::{KgfxKey, KgfxKeyAction, KgfxKeyEvent};

pub mod window;
pub mod graphics;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_init() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shutdown() -> () {
}

pub fn kgfx_is_key_pressed(event: KgfxKeyEvent, key: KgfxKey) -> bool {
    event.key == key && event.action == KgfxKeyAction::Press
}

pub fn kgfx_is_key_released(event: KgfxKeyEvent, key: KgfxKey) -> bool {
    event.key == key && event.action == KgfxKeyAction::Release
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed_i32(event: KgfxKeyEvent, key: i32) -> bool {
    matches!(KgfxKey::from_i32(key), Some(k) if kgfx_is_key_pressed(event, k))
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_released_i32(event: KgfxKeyEvent, key: i32) -> bool {
    matches!(KgfxKey::from_i32(key), Some(k) if kgfx_is_key_released(event, k))
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
