use gl;

use crate::graphics::device::BufferUsage;

pub(crate) struct OpenGLBuffer {
    pub(super) id: u32,
}

impl OpenGLBuffer {
    pub(crate) fn destroy(&mut self) {
        if self.id != 0 {
            unsafe {
                gl::DeleteBuffers(1, &self.id);
            }
            self.id = 0;
        }
    }
}

pub(super) fn create_buffer_init(data: &[f32], usage: BufferUsage) -> Result<OpenGLBuffer, String> {
    if data.is_empty() {
        return Err("buffer data is empty".to_string());
    }
    let target = match usage {
        BufferUsage::Vertex => gl::ARRAY_BUFFER,
    };

    let mut id = 0;
    unsafe {
        gl::GenBuffers(1, &mut id);
        gl::BindBuffer(target, id);
        gl::BufferData(
            target,
            (data.len() * std::mem::size_of::<f32>()) as isize,
            data.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(target, 0);
    }

    Ok(OpenGLBuffer { id })
}
