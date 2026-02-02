use crate::window::{KgfxKeyAction, KgfxKeyEvent};

pub mod window;

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_init() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shutdown() -> () {
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_is_key_pressed(event: KgfxKeyEvent, key: i32) -> bool {
	event.key == key && event.action == KgfxKeyAction::Press
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
