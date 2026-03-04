use crate::graphics::error::GraphicsError;
use crate::window::Window;

use super::backends;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GraphicsApi {
    OpenGL,
    Vulkan,
    DirectX,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ClearColor {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}

pub struct PipelineDescriptor<'a> {
    pub shader: &'a Shader,
}

pub struct ShaderDescriptor<'a> {
    pub vertex_source_glsl: &'a str,
    pub fragment_source_glsl: &'a str,
}

pub struct Buffer {
    pub(crate) inner: BufferInner,
}

pub struct Pipeline {
    pub(crate) inner: PipelineInner,
}

pub struct Shader {
    pub(crate) inner: ShaderInner,
}

pub struct Graphics {
    api: GraphicsApi,
    inner: GraphicsInner,
}

enum GraphicsInner {
    OpenGL(backends::opengl::OpenGLGraphics),
    Vulkan(backends::vulkan::VulkanGraphics),
    DirectX,
}

pub(crate) enum BufferInner {
    OpenGL(backends::opengl::OpenGLBuffer),
    Vulkan(backends::vulkan::VulkanBuffer),
}

pub(crate) enum ShaderInner {
    OpenGL(Arc<backends::opengl::OpenGLShader>),
    Vulkan(Arc<backends::vulkan::VulkanShader>),
}

pub(crate) enum PipelineInner {
    OpenGL(backends::opengl::OpenGLPipeline),
    Vulkan(backends::vulkan::VulkanPipeline),
}

impl Graphics {
    pub fn create(window: &mut Window, api: GraphicsApi) -> Result<Self, GraphicsError> {
        let inner = match api {
            GraphicsApi::OpenGL => GraphicsInner::OpenGL(
                backends::opengl::OpenGLGraphics::create(window).map_err(GraphicsError::from)?,
            ),
            GraphicsApi::Vulkan => GraphicsInner::Vulkan(
                backends::vulkan::VulkanGraphics::create(window).map_err(GraphicsError::from)?,
            ),
            GraphicsApi::DirectX => GraphicsInner::DirectX,
        };

        Ok(Self { api, inner })
    }

    pub fn api(&self) -> GraphicsApi {
        self.api
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        match &mut self.inner {
            GraphicsInner::OpenGL(gfx) => gfx.set_viewport(x, y, width, height),
            GraphicsInner::Vulkan(gfx) => gfx.set_viewport(x, y, width, height),
            GraphicsInner::DirectX => {}
        }
    }

    pub fn create_buffer_init(&mut self, data: &[f32], usage: BufferUsage) -> Result<Buffer, GraphicsError> {
        let inner = match &mut self.inner {
            GraphicsInner::OpenGL(gfx) => BufferInner::OpenGL(
                gfx.create_buffer_init(data, usage).map_err(GraphicsError::from)?,
            ),
            GraphicsInner::Vulkan(gfx) => BufferInner::Vulkan(
                gfx.create_buffer_init(data, usage).map_err(GraphicsError::from)?,
            ),
            GraphicsInner::DirectX => {
                return Err(GraphicsError("DirectX backend not implemented".to_string()));
            }
        };
        Ok(Buffer { inner })
    }

    pub fn create_shader(&mut self, desc: ShaderDescriptor<'_>) -> Result<Shader, GraphicsError> {
        let inner = match &mut self.inner {
            GraphicsInner::OpenGL(gfx) => ShaderInner::OpenGL(
                gfx.create_shader(desc).map_err(GraphicsError::from)?,
            ),
            GraphicsInner::Vulkan(gfx) => ShaderInner::Vulkan(
                gfx.create_shader(desc).map_err(GraphicsError::from)?,
            ),
            GraphicsInner::DirectX => {
                return Err(GraphicsError("DirectX backend not implemented".to_string()));
            }
        };

        Ok(Shader { inner })
    }

    pub fn create_pipeline(&mut self, desc: PipelineDescriptor<'_>) -> Result<Pipeline, GraphicsError> {
        let inner = match (&mut self.inner, &desc.shader.inner) {
            (GraphicsInner::OpenGL(gfx), ShaderInner::OpenGL(shader)) => PipelineInner::OpenGL(
                gfx.create_pipeline(shader).map_err(GraphicsError::from)?,
            ),
            (GraphicsInner::Vulkan(gfx), ShaderInner::Vulkan(shader)) => PipelineInner::Vulkan(
                gfx.create_pipeline(shader).map_err(GraphicsError::from)?,
            ),
            (GraphicsInner::DirectX, _) => {
                return Err(GraphicsError("DirectX backend not implemented".to_string()));
            }
            _ => {
                return Err(GraphicsError(
                    "Shader was created for a different backend".to_string(),
                ));
            }
        };
        Ok(Pipeline { inner })
    }

    pub fn begin_frame(&mut self, window: &mut Window, clear: ClearColor) -> Result<(), GraphicsError> {
        match &mut self.inner {
            GraphicsInner::OpenGL(gfx) => gfx.begin_frame(window, clear).map_err(GraphicsError::from),
            GraphicsInner::Vulkan(gfx) => gfx.begin_frame(window, clear).map_err(GraphicsError::from),
            GraphicsInner::DirectX => Err(GraphicsError("DirectX backend not implemented".to_string())),
        }
    }

    pub fn set_pipeline(&mut self, pipeline: &Pipeline) -> Result<(), GraphicsError> {
        match (&mut self.inner, &pipeline.inner) {
            (GraphicsInner::OpenGL(gfx), PipelineInner::OpenGL(p)) => gfx.set_pipeline(p).map_err(GraphicsError::from),
            (GraphicsInner::Vulkan(gfx), PipelineInner::Vulkan(p)) => gfx.set_pipeline(p).map_err(GraphicsError::from),
            _ => Err(GraphicsError("Pipeline was created for a different backend".to_string())),
        }
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &Buffer) -> Result<(), GraphicsError> {
        match (&mut self.inner, &buffer.inner) {
            (GraphicsInner::OpenGL(gfx), BufferInner::OpenGL(b)) => {
                gfx.set_vertex_buffer(slot, b).map_err(GraphicsError::from)
            }
            (GraphicsInner::Vulkan(gfx), BufferInner::Vulkan(b)) => {
                gfx.set_vertex_buffer(slot, b).map_err(GraphicsError::from)
            }
            _ => Err(GraphicsError("Buffer was created for a different backend".to_string())),
        }
    }

    pub fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), GraphicsError> {
        match &mut self.inner {
            GraphicsInner::OpenGL(gfx) => gfx.draw(vertex_count, first_vertex).map_err(GraphicsError::from),
            GraphicsInner::Vulkan(gfx) => gfx.draw(vertex_count, first_vertex).map_err(GraphicsError::from),
            GraphicsInner::DirectX => Err(GraphicsError("DirectX backend not implemented".to_string())),
        }
    }

    pub fn end_frame(&mut self, window: &mut Window) -> Result<(), GraphicsError> {
        match &mut self.inner {
            GraphicsInner::OpenGL(gfx) => gfx.end_frame(window).map_err(GraphicsError::from),
            GraphicsInner::Vulkan(gfx) => gfx.end_frame(window).map_err(GraphicsError::from),
            GraphicsInner::DirectX => Err(GraphicsError("DirectX backend not implemented".to_string())),
        }
    }

    /// Graceful shutdown for backends that depend on the window message pump
    /// (notably Vulkan WSI). Prefer calling this before dropping `Graphics`.
    pub fn shutdown(&mut self, window: &mut Window) -> Result<(), GraphicsError> {
        match &mut self.inner {
            GraphicsInner::OpenGL(_gfx) => Ok(()),
            GraphicsInner::Vulkan(gfx) => gfx.shutdown(window).map_err(GraphicsError::from),
            GraphicsInner::DirectX => Ok(()),
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        match &mut self.inner {
            BufferInner::OpenGL(b) => b.destroy(),
            BufferInner::Vulkan(b) => b.destroy(),
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        match &mut self.inner {
            PipelineInner::OpenGL(p) => p.destroy(),
            PipelineInner::Vulkan(p) => p.destroy(),
        }
    }
}
