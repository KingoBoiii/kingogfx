use crate::{graphics::{backend::GraphicsBackend, backends::opengl::OpenGLGraphicsBackend, error::GraphicsError}, window::Window};

pub mod error;

pub(crate) mod backend;
pub(crate) mod backends;

pub struct Graphics {
    backend: Box<dyn GraphicsBackend>,
}

impl Graphics {
    pub fn create(window: &mut Window) -> Result<Self, GraphicsError> {
        let backend = OpenGLGraphicsBackend::create(window)
            .map_err(GraphicsError::from)?;
        Ok(Graphics {
            backend: Box::new(backend),
        })
    }

    pub fn clear(&self) {
        self.backend.clear();
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.backend.clear_color(red, green, blue, alpha);
    }
}