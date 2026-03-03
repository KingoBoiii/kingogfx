use crate::{graphics::{backend::GraphicsBackend, error::GraphicsError, pipeline::{Pipeline, PipelineBackend}, shader::{Shader, ShaderBackend}, vertex_buffer::{VertexBuffer, VertexBufferBackend}}, window::Window};

pub mod error;
pub mod shader;
pub mod pipeline;
pub mod vertex_buffer;
pub(crate) mod backend;
pub(crate) mod backends;

pub enum GraphicsApi {
    OpenGL,
    Vulkan,
    DirectX,
    // osv.
}

pub struct Graphics {
	api: GraphicsApi,
	backend: Box<dyn GraphicsBackend>,
}

impl Graphics {
	pub fn create(window: &mut Window, api: GraphicsApi) -> Result<Self, GraphicsError> {
		let backend: Box<dyn GraphicsBackend> = match api {
			GraphicsApi::OpenGL => {
				Box::new(backends::opengl::opengl_backend::OpenGLGraphicsBackend::create(window)
					.map_err(GraphicsError::from)?)
			}
			GraphicsApi::Vulkan => {
				// Tilføj din Vulkan backend her
				unimplemented!("Vulkan backend not yet implemented")
			}
			GraphicsApi::DirectX => {
				// Tilføj din DirectX backend her
				unimplemented!("DirectX backend not yet implemented")
			}
			// osv.
		};

		Ok(Graphics {
			api,
			backend,
		})
	}

	pub fn create_shader(&self, vertex_source: &str, fragment_source: &str) -> Result<Shader, GraphicsError> {
		let backend: Box<dyn ShaderBackend> = match self.api {
			GraphicsApi::OpenGL => {
				Box::new(backends::opengl::opengl_shader::OpenGLShader::from_source(vertex_source, fragment_source)
					.map_err(GraphicsError::from)?)
			}
			GraphicsApi::Vulkan => {
				// Tilføj din Vulkan backend her
				unimplemented!("Vulkan backend not yet implemented")
			}
			GraphicsApi::DirectX => {
				// Tilføj din DirectX backend her
				unimplemented!("DirectX backend not yet implemented")
			}
			// osv.
		};

		Ok(Shader::create(backend))
	}

	pub fn create_pipeline(&self) -> Result<Pipeline, GraphicsError> {
		let backend: Box<dyn PipelineBackend> = match self.api {
			GraphicsApi::OpenGL => {
				Box::new(backends::opengl::opengl_pipeline::OpenGLPipeline::new()
					.map_err(GraphicsError::from)?)
			}
			GraphicsApi::Vulkan => {
				// Tilføj din Vulkan backend her
				unimplemented!("Vulkan backend not yet implemented")
			}
			GraphicsApi::DirectX => {
				// Tilføj din DirectX backend her
				unimplemented!("DirectX backend not yet implemented")
			}
			// osv.
		};

		Ok(Pipeline::create(backend))
	}

	pub fn create_vertex_buffer(&self, data: &[f32]) -> Result<VertexBuffer, GraphicsError> {
		let backend: Box<dyn VertexBufferBackend> = match self.api {
			GraphicsApi::OpenGL => {
				Box::new(backends::opengl::opengl_vertex_buffer::OpenGLVertexBuffer::new(data)
					.map_err(GraphicsError::from)?)
			}
			GraphicsApi::Vulkan => {
				// Tilføj din Vulkan backend her
				unimplemented!("Vulkan backend not yet implemented")
			}
			GraphicsApi::DirectX => {
				// Tilføj din DirectX backend her
				unimplemented!("DirectX backend not yet implemented")
			}
			// osv.
		};
		
		Ok(VertexBuffer::create(backend))
	}

	pub fn clear(&self) {
		self.backend.clear();
	}

	pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
		self.backend.clear_color(red, green, blue, alpha);
	}

	pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
		self.backend.viewport(x, y, width, height);
	}

	pub fn draw_arrays(&self, count: i32) {
		self.backend.draw_arrays(count);
	}
}
