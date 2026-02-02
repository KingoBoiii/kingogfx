#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear() -> () {
  unsafe {
    gl::Clear(gl::COLOR_BUFFER_BIT);
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear_color(red: f32, green: f32, blue: f32, alpha: f32) -> () {
  unsafe {
    gl::ClearColor(red, green, blue, alpha);
  }
}