use glfw::Context;

use crate::{graphics::{KgfxPipelineDesc, KgfxStatus, PipelineBackend, PipelineInner, buffer::{BufferBackend, BufferInner, KgfxBufferDesc, KgfxBufferUsage}}, window::handle::WindowHandle};
use std::{ffi::c_void};

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

// ==================== PIPELINE ====================

pub struct OpenGLPipeline {
  pub id: u32,
}

impl OpenGLPipeline {
	pub fn new(_desc: KgfxPipelineDesc) -> Result<Self, KgfxStatus> {
		let mut id = 0u32;
		
		unsafe {
			gl::GenVertexArrays(1, &mut id);
			if id == 0 {
				return Err(KgfxStatus::InitFailed);
			}

			gl::BindVertexArray(id);
		}

		Ok(Self {
			id,
		})
	}

	pub fn bind(&mut self) -> () {
		unsafe {
			gl::BindVertexArray(self.id);
		}
	}

	pub fn unbind(&mut self) -> () {
		unsafe {
			gl::BindVertexArray(0);
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