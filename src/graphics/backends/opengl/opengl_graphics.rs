use gl;

use std::sync::Arc;

use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
use crate::window::Window;

use super::opengl_buffer;
use super::opengl_pipeline;
use super::opengl_shader;
use super::{OpenGLBuffer, OpenGLPipeline, OpenGLShader};

pub(crate) struct OpenGLGraphics {
    vao: u32,
    in_frame: bool,
}

impl OpenGLGraphics {
    pub(crate) fn create(window: &mut Window) -> Result<Self, String> {
        window.make_current();
        gl::load_with(|s| window.get_proc_address(s));

        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        Ok(Self { vao, in_frame: false })
    }

    pub(crate) fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl::Viewport(x, y, width, height);
        }
    }

    pub(crate) fn create_buffer_init(&mut self, data: &[f32], usage: BufferUsage) -> Result<OpenGLBuffer, String> {
        opengl_buffer::create_buffer_init(data, usage)
    }

    pub(crate) fn create_shader(&mut self, desc: ShaderDescriptor<'_>) -> Result<Arc<OpenGLShader>, String> {
        let shader = opengl_shader::OpenGLShader::from_source(desc.vertex_source_glsl, desc.fragment_source_glsl)?;
        Ok(Arc::new(shader))
    }

    pub(crate) fn create_pipeline(&mut self, shader: &Arc<OpenGLShader>) -> Result<OpenGLPipeline, String> {
        opengl_pipeline::create_pipeline(shader)
    }

    pub(crate) fn begin_frame(&mut self, _window: &mut Window, clear: ClearColor) -> Result<(), String> {
        if self.in_frame {
            return Err("begin_frame called while already in a frame".to_string());
        }

        unsafe {
            gl::ClearColor(clear.r, clear.g, clear.b, clear.a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.in_frame = true;
        Ok(())
    }

    pub(crate) fn set_pipeline(&mut self, pipeline: &OpenGLPipeline) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_pipeline must be called between begin_frame/end_frame".to_string());
        }

        unsafe {
            gl::UseProgram(pipeline.shader.program_id());
            gl::BindVertexArray(self.vao);
        }

        Ok(())
    }

    pub(crate) fn set_vertex_buffer(&mut self, slot: u32, buffer: &OpenGLBuffer) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_vertex_buffer must be called between begin_frame/end_frame".to_string());
        }
        if slot != 0 {
            return Err("OpenGL backend currently supports only slot 0".to_string());
        }

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer.id);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * std::mem::size_of::<f32>()) as i32,
                0 as *const _,
            );
        }

        Ok(())
    }

    pub(crate) fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), String> {
        if !self.in_frame {
            return Err("draw must be called between begin_frame/end_frame".to_string());
        }
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, first_vertex as i32, vertex_count as i32);
        }
        Ok(())
    }

    pub(crate) fn end_frame(&mut self, window: &mut Window) -> Result<(), String> {
        if !self.in_frame {
            return Err("end_frame called without begin_frame".to_string());
        }
        window.swap_buffers();
        self.in_frame = false;
        Ok(())
    }
}
