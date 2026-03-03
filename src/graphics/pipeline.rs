pub trait PipelineBackend {
	fn bind(&self);
	fn unbind(&self);
}

pub struct Pipeline {
	backend: Box<dyn PipelineBackend>,
}

impl Pipeline {
	pub fn create(backend: Box<dyn PipelineBackend>) -> Self {
		Pipeline { backend }
	}

	pub fn bind(&self) {
		self.backend.bind();
	}

	pub fn unbind(&self) {
		self.backend.unbind();
	}
}