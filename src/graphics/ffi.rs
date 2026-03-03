use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_float, c_void};

use super::{Graphics, GraphicsApi, Shader, Pipeline, VertexBuffer};
use crate::window::Window;

#[repr(C)]
pub struct KgfxGraphics {
    _private: [u8; 0],
}
#[repr(C)]
pub struct KgfxShader {
    _private: [u8; 0],
}
#[repr(C)]
pub struct KgfxPipeline {
    _private: [u8; 0],
}
#[repr(C)]
pub struct KgfxVertexBuffer {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KgfxStatus {
    Ok,
    Error,
}
#[repr(C)]
pub enum KgfxApi {
    OpenGL,
    Vulkan,
    DirectX,
}

fn as_graphics_mut<'a>(handle: *mut KgfxGraphics) -> Option<&'a mut Graphics> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Graphics>() })
    }
}
fn as_shader_mut<'a>(handle: *mut KgfxShader) -> Option<&'a mut Shader> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Shader>() })
    }
}
fn as_pipeline_mut<'a>(handle: *mut KgfxPipeline) -> Option<&'a mut Pipeline> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Pipeline>() })
    }
}
fn as_vertex_buffer_mut<'a>(handle: *mut KgfxVertexBuffer) -> Option<&'a mut VertexBuffer> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<VertexBuffer>() })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create(
    window: *mut c_void,
    api: KgfxApi,
    out_graphics: *mut *mut KgfxGraphics,
) -> KgfxStatus {
    let window = unsafe { &mut *(window as *mut Window) };
    let api = match api {
        KgfxApi::OpenGL => GraphicsApi::OpenGL,
        KgfxApi::Vulkan => GraphicsApi::Vulkan,
        KgfxApi::DirectX => GraphicsApi::DirectX,
    };
    match Graphics::create(window, api) {
        Ok(gfx) => {
            unsafe { *out_graphics = Box::into_raw(Box::new(gfx)).cast(); }
            KgfxStatus::Ok
        }
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy(graphics: *mut KgfxGraphics) {
    if !graphics.is_null() {
        unsafe { drop(Box::from_raw(graphics.cast::<Graphics>())); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear(graphics: *mut KgfxGraphics) {
    let Some(gfx) = as_graphics_mut(graphics) else { return };
    gfx.clear();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear_color(
    graphics: *mut KgfxGraphics,
    r: c_float,
    g: c_float,
    b: c_float,
    a: c_float,
) {
    let Some(gfx) = as_graphics_mut(graphics) else { return };
    gfx.clear_color(r, g, b, a);
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_viewport(
    graphics: *mut KgfxGraphics,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
) {
    let Some(gfx) = as_graphics_mut(graphics) else { return };
    gfx.viewport(x, y, width, height);
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_draw_arrays(
    graphics: *mut KgfxGraphics,
    count: c_int,
) {
    let Some(gfx) = as_graphics_mut(graphics) else { return };
    gfx.draw_arrays(count);
}

// Shader
#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_shader(
    graphics: *mut KgfxGraphics,
    vertex_source: *const c_char,
    fragment_source: *const c_char,
    out_shader: *mut *mut KgfxShader,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    let vertex_source = unsafe {
        CStr::from_ptr(vertex_source).to_str().unwrap_or("").to_string()
    };
    let fragment_source = unsafe {
        CStr::from_ptr(fragment_source).to_str().unwrap_or("").to_string()
    };
    match gfx.create_shader(&vertex_source, &fragment_source) {
        Ok(shader) => {
            unsafe { *out_shader = Box::into_raw(Box::new(shader)).cast(); }
            KgfxStatus::Ok
        }
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shader_destroy(shader: *mut KgfxShader) {
    if !shader.is_null() {
        unsafe { drop(Box::from_raw(shader.cast::<Shader>())); }
    }
}

// Pipeline
#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_pipeline(
    graphics: *mut KgfxGraphics,
    out_pipeline: *mut *mut KgfxPipeline,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    match gfx.create_pipeline() {
        Ok(pipeline) => {
            unsafe { *out_pipeline = Box::into_raw(Box::new(pipeline)).cast(); }
            KgfxStatus::Ok
        }
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_pipeline_destroy(pipeline: *mut KgfxPipeline) {
    if !pipeline.is_null() {
        unsafe { drop(Box::from_raw(pipeline.cast::<Pipeline>())); }
    }
}

// Vertex Buffer
#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_vertex_buffer(
    graphics: *mut KgfxGraphics,
    data: *const c_float,
    len: usize,
    out_buffer: *mut *mut KgfxVertexBuffer,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    if data.is_null() || len == 0 {
        return KgfxStatus::Error;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    match gfx.create_vertex_buffer(slice) {
        Ok(buffer) => {
            unsafe { *out_buffer = Box::into_raw(Box::new(buffer)).cast(); }
            KgfxStatus::Ok
        }
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_vertex_buffer_destroy(buffer: *mut KgfxVertexBuffer) {
    if !buffer.is_null() {
        unsafe { drop(Box::from_raw(buffer.cast::<VertexBuffer>())); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_bind_shader(
    shader: *mut KgfxShader,
) {
    let Some(shader) = as_shader_mut(shader) else { return };
    shader.bind();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_bind_pipeline(
    pipeline: *mut KgfxPipeline,
) {
    let Some(pipeline) = as_pipeline_mut(pipeline) else { return };
    pipeline.bind();
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_bind_vertex_buffer(
    buffer: *mut KgfxVertexBuffer,
) {
    let Some(buffer) = as_vertex_buffer_mut(buffer) else { return };
    buffer.bind();
}
