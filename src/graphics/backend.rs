pub(crate) trait GraphicsBackend {
  fn clear(&self);
  fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32);
}