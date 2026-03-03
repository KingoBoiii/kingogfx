use crate::graphics::vertex_buffer::VertexBufferBackend;

pub struct OpenGLVertexBuffer {
    id: u32,
}

impl OpenGLVertexBuffer {
  pub fn new(vertices: &[f32]) -> Result<Self, String> {
    let mut id = 0;
    unsafe {
      gl::GenBuffers(1, &mut id);
      gl::BindBuffer(gl::ARRAY_BUFFER, id);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (vertices.len() * std::mem::size_of::<f32>()) as isize,
        vertices.as_ptr() as *const _,
        gl::STATIC_DRAW,
      );
    }
    Ok(OpenGLVertexBuffer { id })
  }
}

impl VertexBufferBackend for OpenGLVertexBuffer {
  fn bind(&self) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
    }
  }
  
  fn unbind(&self) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
  }
}