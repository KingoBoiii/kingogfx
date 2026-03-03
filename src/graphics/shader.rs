pub trait ShaderBackend {
    fn bind(&self);
    fn unbind(&self);
}

pub struct Shader {
    backend: Box<dyn ShaderBackend>,
}

impl Shader {
    pub fn create(backend: Box<dyn ShaderBackend>) -> Self {
        Shader { backend }
    }

    pub fn bind(&self) {
        self.backend.bind();
    }

    pub fn unbind(&self) {
        self.backend.unbind();
    }
}