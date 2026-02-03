use glfw::Context;

use crate::{graphics::{BufferBackend, BufferInner, KgfxBufferDesc, KgfxBufferUsage, KgfxPipelineDesc, KgfxStatus, PipelineBackend, PipelineInner, ShaderBackend, ShaderInner}, window::handle::WindowHandle};
use std::{
	ffi::{CString, c_void},
	mem::size_of
};

// ==================== BACKEND ====================

pub struct OpenGLBackend;

impl OpenGLBackend {
	pub fn new(window: &mut WindowHandle) -> Option<Self> {
		// GLFW/OpenGL: context skal være current før du loader pointers
		window.window.make_current();

		gl::load_with(|symbol| match window.window.get_proc_address(symbol) {
			Some(proc_addr) => proc_addr as *const c_void,
			None => std::ptr::null(),
		});

		Some(Self)
	}

	pub fn draw_arrays(&mut self, pipeline: &mut OpenGLPipeline, count: i32) -> () {
		unsafe {
			pipeline.bind();
			gl::DrawArrays(gl::TRIANGLES, 0, count);
			pipeline.unbind();
		}
	}

	pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) -> () {
		unsafe {
			gl::Viewport(x, y, width, height);
		}
	}

	pub fn clear(&mut self) -> () {
		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
	}

	pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> () {
		unsafe {
			gl::ClearColor(r, g, b, a);
		}
	}
}

// ==================== SHADER ====================

pub struct OpenGLShader {
	pub id: u32,
}

impl OpenGLShader {
	pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Result<Self, KgfxStatus> {
		let vertex_shader = match Self::compile_shader(vertex_shader_source, gl::VERTEX_SHADER) {
			Some(s) => s,
			None => return Err(KgfxStatus::InitFailed)
		};

		let fragment_shader = match Self::compile_shader(fragment_shader_source, gl::FRAGMENT_SHADER) {
			Some(s) => s,
			None => {
				unsafe {
					gl::DeleteShader(vertex_shader);
				}
				return Err(KgfxStatus::InitFailed)
			} 
		};
		
		let program = match Self::link_program(vertex_shader, fragment_shader) {
			Some(p) => p,
			None => {
				unsafe {
					gl::DeleteShader(vertex_shader);
					gl::DeleteShader(fragment_shader);
				}
				return Err(KgfxStatus::InitFailed);
			}
		};

		Ok(Self {
			id: program,
		})
	}

	pub fn bind(&self) -> () {
		unsafe {
			gl::UseProgram(self.id);
		}
	}

	pub fn unbind(&self) -> () {
		unsafe {
			gl::UseProgram(0);
		}
	}

	fn compile_shader(src: &str, shader_type: u32) -> Option<u32> {
		unsafe {
			let shader = gl::CreateShader(shader_type);
			
			let c_str = CString::new(src).ok()?;
			gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
			gl::CompileShader(shader);

			let mut ok = 0;
			gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
			if ok == 0 {
					gl::DeleteShader(shader);
					return None;
			}
			
			return Some(shader);
		}
	}

	fn link_program(vertex_shader: u32, fragment_shader: u32) -> Option<u32> {
		unsafe {
			let program = gl::CreateProgram();
			
			gl::AttachShader(program, vertex_shader);
			gl::AttachShader(program, fragment_shader);

			gl::LinkProgram(program);

			let mut ok = 0;
			gl::GetProgramiv(program, gl::LINK_STATUS, &mut ok);
			if ok == 0 {
					gl::DeleteProgram(program);
					return None;
			}
			
			return Some(program);
		}
	}
}

impl Drop for ShaderInner {
  fn drop(&mut self) {
    match &mut self.backend {
      ShaderBackend::OpenGL(b) => unsafe {
        if b.id != 0 {
					gl::DeleteProgram(b.id);
          b.id = 0;
        }
      },
    }
  }
}

// ==================== PIPELINE ====================

pub struct OpenGLPipeline {
  pub id: u32,
	pub(crate) shader: *mut ShaderInner
}

impl OpenGLPipeline {
	pub fn new(desc: KgfxPipelineDesc) -> Result<Self, KgfxStatus> {
		let mut id = 0u32;
		
		unsafe {
			gl::GenVertexArrays(1, &mut id);
			if id == 0 {
				return Err(KgfxStatus::InitFailed);
			}

			gl::BindVertexArray(id);
		}

		let shader_ptr = desc.shader as *mut ShaderInner;
		unsafe {
			let shader_inner = &mut *shader_ptr;
			match &shader_inner.backend {
				ShaderBackend::OpenGL(_) => {}
			}
		}

		// KgfxShader handle -> ShaderInner -> (kopiér OpenGLShader ud)
		// let shader = unsafe {
		// 		let shader_inner = &mut *(desc.shader as *mut ShaderInner);
		// 		match &shader_inner.backend {
		// 				ShaderBackend::OpenGL(gl_shader) => *gl_shader, // Copy
		// 				_ => return Err(KgfxStatus::Unsupported),
		// 		}
		// };

		Ok(Self {
			id,
			shader: shader_ptr
		})
	}

	pub fn bind(&mut self) -> () {
		unsafe {
			let shader_inner = &mut *self.shader;
			match &shader_inner.backend {
				ShaderBackend::OpenGL(gl_shader) => gl_shader.bind(),
			}

			gl::BindVertexArray(self.id);
		}
	}

	pub fn unbind(&mut self) -> () {
		unsafe {
			gl::BindVertexArray(0);
			gl::UseProgram(0);
		}
	}
}

impl Drop for PipelineInner {
  fn drop(&mut self) {
    match &mut self.backend {
      PipelineBackend::OpenGL(b) => unsafe {
        if b.id != 0 {
          gl::DeleteVertexArrays(1, &b.id);
          b.id = 0;
        }
      },
    }
  }
}

// ==================== BUFFER ====================

pub struct OpenGLBuffer {
  pub id: u32,
  pub target: u32,
  pub size_bytes: usize,
}

impl OpenGLBuffer {
	pub fn new(desc: KgfxBufferDesc, initial_data: *const u8) -> Result<Self, KgfxStatus> {
		let target = match desc.usage {
			KgfxBufferUsage::Vertex => gl::ARRAY_BUFFER,
			KgfxBufferUsage::Index => gl::ELEMENT_ARRAY_BUFFER,
			KgfxBufferUsage::Uniform => gl::UNIFORM_BUFFER,
		};

		let mut id = 0u32;

		unsafe {
			gl::GenBuffers(1, &mut id);
			if id == 0 {
				return Err(KgfxStatus::InitFailed);
			}

			gl::BindBuffer(target, id);

			let data_ptr = if initial_data.is_null() {
				std::ptr::null()
			} else {
				initial_data as *const c_void
			};

			gl::BufferData(
				target,
				desc.size_bytes as isize,
				data_ptr,
				gl::STATIC_DRAW,
			);

			let stride = (2 * size_of::<f32>()) as i32;
			gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
			gl::EnableVertexAttribArray(0);
		}

		Ok(Self {
			id,
			target,
			size_bytes: desc.size_bytes,
		})
	}
}

impl Drop for BufferInner {
  fn drop(&mut self) {
    match &mut self.backend {
      BufferBackend::OpenGL(b) => unsafe {
        if b.id != 0 {
          gl::DeleteBuffers(1, &b.id);
          b.id = 0;
        }
      },
    }
  }
}