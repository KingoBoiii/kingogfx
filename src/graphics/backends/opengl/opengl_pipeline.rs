use std::sync::Arc;

use super::OpenGLShader;

pub(crate) struct OpenGLPipeline {
    pub(super) shader: Arc<OpenGLShader>,
}

impl OpenGLPipeline {
    pub(crate) fn destroy(&mut self) {
    }
}

pub(super) fn create_pipeline(shader: &Arc<OpenGLShader>) -> Result<OpenGLPipeline, String> {
    Ok(OpenGLPipeline {
        shader: Arc::clone(shader),
    })
}
