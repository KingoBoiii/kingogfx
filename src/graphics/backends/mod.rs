use crate::graphics::backends::opengl::{OpenGLBuffer, OpenGLPipeline, OpenGLShader};

pub mod opengl;

// ==================== SHADER ====================

pub(crate) enum ShaderBackend {
  OpenGL(OpenGLShader),
  // Vulkan(...), Dx11(...), Dx12(...)
}

pub(crate) struct ShaderInner {
  pub backend: ShaderBackend,
}

// ==================== PIPELINE ====================

pub(crate) enum PipelineBackend {
  OpenGL(OpenGLPipeline),
  // Vulkan(...), Dx11(...), Dx12(...)
}

pub(crate) struct PipelineInner {
  pub backend: PipelineBackend,
}

// ==================== BUFFER ====================

pub(crate) enum BufferBackend {
  OpenGL(OpenGLBuffer),
  // Vulkan(...), Dx11(...), Dx12(...)
}

pub(crate) struct BufferInner {
  pub backend: BufferBackend,
}