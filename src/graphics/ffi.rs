use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_float, c_void};

use super::{
    Buffer,
    BufferUsage,
    ClearColor,
    Graphics,
    GraphicsApi,
    Pipeline,
    PipelineDescriptor,
    Shader,
    ShaderDescriptor,
    ShaderSource,
};
use crate::window::Window;

#[repr(C)]
pub struct KgfxGraphics {
    _private: [u8; 0],
}
#[repr(C)]
pub struct KgfxPipeline {
    _private: [u8; 0],
}
#[repr(C)]
pub struct KgfxShader {
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
    DirectX11,
    DirectX12,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KgfxShaderLanguage {
    Glsl,
    Hlsl,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KgfxShaderCreateDesc {
    pub vertex_language: KgfxShaderLanguage,
    pub vertex_source: *const c_char,
    pub fragment_language: KgfxShaderLanguage,
    pub fragment_source: *const c_char,
}

fn as_graphics_mut<'a>(handle: *mut KgfxGraphics) -> Option<&'a mut Graphics> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Graphics>() })
    }
}
fn as_pipeline_mut<'a>(handle: *mut KgfxPipeline) -> Option<&'a mut Pipeline> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Pipeline>() })
    }
}
fn as_shader_mut<'a>(handle: *mut KgfxShader) -> Option<&'a mut Shader> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Shader>() })
    }
}
fn as_vertex_buffer_mut<'a>(handle: *mut KgfxVertexBuffer) -> Option<&'a mut Buffer> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *handle.cast::<Buffer>() })
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
        KgfxApi::DirectX11 => GraphicsApi::DirectX11,
        KgfxApi::DirectX12 => GraphicsApi::DirectX12,
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
pub extern "C" fn kgfx_graphics_shutdown(graphics: *mut KgfxGraphics, window: *mut c_void) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    if window.is_null() {
        return KgfxStatus::Error;
    }
    let window = unsafe { &mut *(window as *mut Window) };
    match gfx.shutdown(window) {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Error,
    }
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
    gfx.set_viewport(x, y, width, height);
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_pipeline(
    graphics: *mut KgfxGraphics,
    shader: *mut KgfxShader,
    out_pipeline: *mut *mut KgfxPipeline,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    let Some(shader) = as_shader_mut(shader) else { return KgfxStatus::Error };

    match gfx.create_pipeline(PipelineDescriptor { shader }) {
        Ok(pipeline) => {
            unsafe { *out_pipeline = Box::into_raw(Box::new(pipeline)).cast(); }
            KgfxStatus::Ok
        }
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_shader(
    graphics: *mut KgfxGraphics,
    desc: *const KgfxShaderCreateDesc,
    out_shader: *mut *mut KgfxShader,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    if desc.is_null() || out_shader.is_null() {
        return KgfxStatus::Error;
    }

    let desc = unsafe { &*desc };
    if desc.vertex_source.is_null() || desc.fragment_source.is_null() {
        return KgfxStatus::Error;
    }

    let vertex_source = unsafe { CStr::from_ptr(desc.vertex_source) }.to_str().unwrap_or("");
    let fragment_source = unsafe { CStr::from_ptr(desc.fragment_source) }.to_str().unwrap_or("");

    let vertex = match desc.vertex_language {
        KgfxShaderLanguage::Glsl => ShaderSource::glsl(vertex_source),
        KgfxShaderLanguage::Hlsl => ShaderSource::hlsl(vertex_source),
    };
    let fragment = match desc.fragment_language {
        KgfxShaderLanguage::Glsl => ShaderSource::glsl(fragment_source),
        KgfxShaderLanguage::Hlsl => ShaderSource::hlsl(fragment_source),
    };

    match gfx.create_shader(ShaderDescriptor { vertex, fragment }) {
        Ok(shader) => {
            unsafe { *out_shader = Box::into_raw(Box::new(shader)).cast(); }
            KgfxStatus::Ok
        }
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_shader_legacy(
    graphics: *mut KgfxGraphics,
    vertex_source: *const c_char,
    fragment_source: *const c_char,
    out_shader: *mut *mut KgfxShader,
) -> KgfxStatus {
    let desc = KgfxShaderCreateDesc {
        vertex_language: KgfxShaderLanguage::Glsl,
        vertex_source,
        fragment_language: KgfxShaderLanguage::Glsl,
        fragment_source,
    };
    kgfx_graphics_create_shader(graphics, &desc, out_shader)
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_shader_destroy(shader: *mut KgfxShader) {
    if !shader.is_null() {
        unsafe { drop(Box::from_raw(shader.cast::<Shader>())); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_pipeline_destroy(pipeline: *mut KgfxPipeline) {
    if !pipeline.is_null() {
        unsafe { drop(Box::from_raw(pipeline.cast::<Pipeline>())); }
    }
}

// Vertex Buffer (float array)
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

    match gfx.create_buffer_init(slice, BufferUsage::Vertex) {
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
        unsafe { drop(Box::from_raw(buffer.cast::<Buffer>())); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_begin_frame(
    graphics: *mut KgfxGraphics,
    window: *mut c_void,
    r: c_float,
    g: c_float,
    b: c_float,
    a: c_float,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    if window.is_null() {
        return KgfxStatus::Error;
    }
    let window = unsafe { &mut *(window as *mut Window) };
    let clear = ClearColor { r, g, b, a };

    match gfx.begin_frame(window, clear) {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_set_pipeline(
    graphics: *mut KgfxGraphics,
    pipeline: *mut KgfxPipeline,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    let Some(p) = as_pipeline_mut(pipeline) else { return KgfxStatus::Error };
    match gfx.set_pipeline(p) {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_set_vertex_buffer(
    graphics: *mut KgfxGraphics,
    slot: u32,
    buffer: *mut KgfxVertexBuffer,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    let Some(b) = as_vertex_buffer_mut(buffer) else { return KgfxStatus::Error };
    match gfx.set_vertex_buffer(slot, b) {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_draw(
    graphics: *mut KgfxGraphics,
    vertex_count: u32,
    first_vertex: u32,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    match gfx.draw(vertex_count, first_vertex) {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_end_frame(
    graphics: *mut KgfxGraphics,
    window: *mut c_void,
) -> KgfxStatus {
    let Some(gfx) = as_graphics_mut(graphics) else { return KgfxStatus::Error };
    if window.is_null() {
        return KgfxStatus::Error;
    }
    let window = unsafe { &mut *(window as *mut Window) };
    match gfx.end_frame(window) {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Error,
    }
}
