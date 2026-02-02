pub mod backends;
pub mod ffi;

pub use ffi::*;

use backends::opengl::OpenGLBackend;

pub enum Backend {
    OpenGL(OpenGLBackend),
    // Vulkan(...),
    // Dx11(...),
    // Dx12(...),
}

pub struct GraphicsContext {
    backend: Backend,
}

impl GraphicsContext {
    pub fn clear(&mut self) {
        match &mut self.backend {
            Backend::OpenGL(glb) => glb.clear(),
        }
    }

    pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        match &mut self.backend {
            Backend::OpenGL(glb) => glb.clear_color(r, g, b, a),
        }
    }
}
