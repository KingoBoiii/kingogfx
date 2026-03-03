pub trait VertexBufferBackend {
  fn bind(&self);
  fn unbind(&self);
}

pub struct VertexBuffer {
  backend: Box<dyn VertexBufferBackend>,
}

impl VertexBuffer {
  pub fn create(backend: Box<dyn VertexBufferBackend>) -> Self {
      VertexBuffer { backend }
  }

  pub fn bind(&self) {
      self.backend.bind();
  }

  pub fn unbind(&self) {
      self.backend.unbind();
  }
}