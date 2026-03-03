use crate::gl::GLProcLoader;
use crate::window::Window;
use crate::graphics::backend::GraphicsBackend;
use gl;

pub(crate) struct OpenGLGraphicsBackend;

impl OpenGLGraphicsBackend {
    pub(crate) fn create(window: &mut Window) -> Result<Self, String> {
			window.make_current();

			gl::load_with(|s| window.get_proc_address(s));

			Ok(OpenGLGraphicsBackend)
    }
}

impl GraphicsBackend for OpenGLGraphicsBackend {
		fn clear(&self) {
			unsafe {
				gl::Clear(gl::COLOR_BUFFER_BIT);
			}
		}

		fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
			unsafe {
				gl::ClearColor(red, green, blue, alpha);
			}
		}

		fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
			unsafe {
				gl::Viewport(x, y, width, height);
			}
		}
}
