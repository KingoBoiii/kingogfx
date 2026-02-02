use crate::graphics::{GraphicsContext, KgfxStatus, backends::opengl::OpenGLBuffer};

#[repr(C)]
pub struct KgfxBuffer { _private: [u8; 0] }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum KgfxBufferUsage {
  Vertex = 1,
  Index = 2,
  Uniform = 3,
}

#[repr(C)]
pub struct KgfxBufferDesc {
  pub struct_size: u32,
  pub usage: KgfxBufferUsage,
  pub size_bytes: usize,
}

pub(crate) enum BufferBackend {
  OpenGL(OpenGLBuffer),
  // Vulkan(...), Dx11(...), Dx12(...)
}

pub(crate) struct BufferInner {
  pub backend: BufferBackend,
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_buffer(ctx: *mut GraphicsContext, desc: KgfxBufferDesc, initial_data: *const u8, out_buffer: *mut *mut KgfxBuffer) -> KgfxStatus {
    if ctx.is_null() || out_buffer.is_null() {
        return KgfxStatus::NullPointer;
    }

    unsafe { *out_buffer = std::ptr::null_mut() };

    if desc.struct_size as usize != std::mem::size_of::<KgfxBufferDesc>() {
        return KgfxStatus::InvalidArg;
    }

    let result = std::panic::catch_unwind(|| unsafe {
        let buffer_ptr = (*ctx).create_buffer(desc, initial_data)?;
        *out_buffer = buffer_ptr;
        Ok::<(), KgfxStatus>(())
    });

    match result {
        Ok(Ok(())) => KgfxStatus::Ok,
        Ok(Err(s)) => s,
        Err(_) => KgfxStatus::Panic,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy_buffer(_ctx: *mut GraphicsContext, buffer: *mut KgfxBuffer) -> KgfxStatus {
    if buffer.is_null() {
        return KgfxStatus::NullPointer;
    }

    let result = std::panic::catch_unwind(|| unsafe {
        // KgfxBuffer er en opaque handle; den peger i praksis på BufferInner.
        drop(Box::from_raw(buffer as *mut BufferInner));
    });

    match result {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Panic,
    }
}

