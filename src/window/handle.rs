extern crate glfw;

use std::collections::VecDeque;

pub struct WindowHandle {
  pub(crate) glfw: glfw::Glfw,
  pub(crate) window: glfw::PWindow,
  pub(crate) events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
  pub(crate) event_queue: VecDeque<glfw::WindowEvent>,
}