use crate::graphics::{pipeline::PipelineBackend};

pub struct OpenGLPipeline {
    id: u32,
}

impl OpenGLPipeline {
  pub fn new() -> Result<Self, String> {
    let mut id: u32 = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut id);
      gl::BindVertexArray(id);

      gl::EnableVertexAttribArray(0);
      gl::VertexAttribPointer(0, 2, gl::FLOAT, false as u8, 2 * std::mem::size_of::<f32>() as i32, 0 as *const _);
    }
    Ok(OpenGLPipeline { id })
  }
}

impl PipelineBackend for OpenGLPipeline {
  fn bind(&self) {
    unsafe {
      gl::BindVertexArray(self.id);
    }
  }

  fn unbind(&self) {
    unsafe {
      gl::BindVertexArray(0);
    }
  }
}