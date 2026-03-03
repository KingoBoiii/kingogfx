pub(crate) trait GraphicsBackend {
  fn clear(&self);
  fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32);
  fn viewport(&self, x: i32, y: i32, width: i32, height: i32);
  fn draw_arrays(&self, count: i32);
}