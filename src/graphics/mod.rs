pub mod backends;
pub mod ffi;
pub mod buffer;

pub use ffi::*;

use backends::opengl::OpenGLBackend;

use crate::graphics::{backends::opengl::{OpenGLBuffer, OpenGLPipeline}, buffer::{BufferBackend, BufferInner, KgfxBuffer, KgfxBufferDesc}};

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
	pub fn create_pipeline(&mut self, desc: KgfxPipelineDesc) -> Result<*mut KgfxPipeline, KgfxStatus> {
		let inner = match &mut self.backend {
			Backend::OpenGL(_) => PipelineInner { backend: PipelineBackend::OpenGL(OpenGLPipeline::new(desc)?) }
		};

		Ok(Box::into_raw(Box::new(inner)) as *mut KgfxPipeline)
	}
	
	pub fn create_buffer(&mut self, desc: KgfxBufferDesc, initial_data: *const u8) -> Result<*mut KgfxBuffer, KgfxStatus> {
		let inner = match &mut self.backend {
			Backend::OpenGL(_) => BufferInner { backend: BufferBackend::OpenGL(OpenGLBuffer::new(desc, initial_data)?) }
		};

		Ok(Box::into_raw(Box::new(inner)) as *mut KgfxBuffer)
	}

	pub fn draw_arrays(&mut self, pipeline: *mut KgfxPipeline, count: i32) -> () {
		if pipeline.is_null() {
				return;
		}

		let pipeline = unsafe { &mut *(pipeline as *mut PipelineInner) };

		match (&mut self.backend, &mut pipeline.backend) {
			(Backend::OpenGL(glb), PipelineBackend::OpenGL(pl)) => glb.draw_arrays(pl, count),
		}
	}

	pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) -> () {
		match &mut self.backend {
			Backend::OpenGL(glb) => glb.viewport(x, y, width, height),
		}
	}

	pub fn clear(&mut self) -> () {
		match &mut self.backend {
			Backend::OpenGL(glb) => glb.clear(),
		}
	}

	pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> () {
		match &mut self.backend {
			Backend::OpenGL(glb) => glb.clear_color(r, g, b, a),
		}
	}
}
