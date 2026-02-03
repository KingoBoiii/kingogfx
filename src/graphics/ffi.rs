use std::ffi::CStr;
use std::os::raw::c_char;

use crate::graphics::backends::opengl::{OpenGLPipeline, OpenGLShader};
use crate::graphics::{Backend, GraphicsContext};
use crate::graphics::backends;
use crate::window::handle::WindowHandle;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum BackendKind {
	OpenGL = 0,
	Vulkan = 1,
	DirectX11 = 2,
	DirectX12 = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KgfxStatus {
	Ok = 0,
	NullPointer = 1,
	Unsupported = 2,
	InitFailed = 3,
	InvalidArg = 4,
	Panic = 255,
}

// Opaque handles (ABI-stabile; ingen layout udadtil)
#[repr(C)]
pub struct KgfxContext { _private: [u8; 0] }

// ==================== CONTEXT ====================

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_context(kind: BackendKind, window_handle: *mut WindowHandle, out_ctx: *mut *mut GraphicsContext) -> KgfxStatus {
	if window_handle.is_null() || out_ctx.is_null() {
		return KgfxStatus::NullPointer;
	}

	// Sæt altid out til null som default (så caller ikke får garbage ved fejl)
	unsafe { *out_ctx = std::ptr::null_mut() };

	let result = std::panic::catch_unwind(|| {
		let window = unsafe { &mut *window_handle };

		let backend = match kind {
			BackendKind::OpenGL => {
				let glb = backends::opengl::OpenGLBackend::new(window).ok_or(KgfxStatus::InitFailed)?;
				Backend::OpenGL(glb)
			}
			_ => return Err(KgfxStatus::Unsupported),
		};

		let ctx = GraphicsContext { backend };
		let ptr = Box::into_raw(Box::new(ctx));
		unsafe { *out_ctx = ptr };

		Ok::<(), KgfxStatus>(())
	});

	match result {
		Ok(Ok(())) => KgfxStatus::Ok,
		Ok(Err(status)) => status,
		Err(_) => KgfxStatus::Panic,
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy_context(ctx: *mut GraphicsContext) -> KgfxStatus {
	if ctx.is_null() {
		return KgfxStatus::NullPointer;
	}

	let result = std::panic::catch_unwind(|| unsafe {
		drop(Box::from_raw(ctx));
	});

	match result {
		Ok(()) => KgfxStatus::Ok,
		Err(_) => KgfxStatus::Panic,
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_draw_arrays(ctx: *mut GraphicsContext, pipeline: *mut KgfxPipeline, count: i32) -> KgfxStatus {
	if ctx.is_null() {
		return KgfxStatus::NullPointer;
	}

	let result = std::panic::catch_unwind(|| unsafe {
		(*ctx).draw_arrays(pipeline, count);
	});

	match result {
		Ok(()) => KgfxStatus::Ok,
		Err(_) => KgfxStatus::Panic,
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_viewport(ctx: *mut GraphicsContext, x: i32, y: i32, width: i32, height: i32) -> KgfxStatus {
	if ctx.is_null() {
		return KgfxStatus::NullPointer;
	}

	let result = std::panic::catch_unwind(|| unsafe {
		(*ctx).viewport(x, y, width, height);
	});

	match result {
		Ok(()) => KgfxStatus::Ok,
		Err(_) => KgfxStatus::Panic,
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear(ctx: *mut GraphicsContext) -> KgfxStatus {
	if ctx.is_null() {
		return KgfxStatus::NullPointer;
	}

	let result = std::panic::catch_unwind(|| unsafe {
		(*ctx).clear();
	});

	match result {
		Ok(()) => KgfxStatus::Ok,
		Err(_) => KgfxStatus::Panic,
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear_color(
	ctx: *mut GraphicsContext,
	red: f32,
	green: f32,
	blue: f32,
	alpha: f32,
) -> KgfxStatus {
	if ctx.is_null() {
		return KgfxStatus::NullPointer;
	}

	let result = std::panic::catch_unwind(|| unsafe {
		(*ctx).clear_color(red, green, blue, alpha);
	});

	match result {
		Ok(()) => KgfxStatus::Ok,
		Err(_) => KgfxStatus::Panic,
	}
}

// ==================== SHADER ====================

#[repr(C)]
pub struct KgfxShader { _private: [u8; 0] }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum KgfxShaderStage {
    Vertex = 1,
    Fragment = 2,
}

#[repr(C)]
pub struct KgfxShaderDesc {
    pub struct_size: u32,
    pub stage: KgfxShaderStage,
    // GLSL/HLSL/SPIR-V senere; start simpelt:
    pub source_utf8: *const u8,
    pub source_len: usize,
}

pub(crate) enum ShaderBackend {
  OpenGL(OpenGLShader),
  // Vulkan(...), Dx11(...), Dx12(...)
}

pub(crate) struct ShaderInner {
  pub backend: ShaderBackend,
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_shader(ctx: *mut GraphicsContext, vertex_shader_source: *const c_char, fragment_shader_source: *const c_char, out_shader: *mut *mut KgfxShader) -> KgfxStatus {
    if ctx.is_null() || out_shader.is_null() {
        return KgfxStatus::NullPointer;
    }

    unsafe { *out_shader = std::ptr::null_mut() };

		if vertex_shader_source.is_null() || fragment_shader_source.is_null() {
				return KgfxStatus::InvalidArg;
		}

		let vertex_shader_source_str = match unsafe { CStr::from_ptr(vertex_shader_source) }.to_str() {
				Ok(s) => s,
				Err(_) => return KgfxStatus::InvalidArg,
		};

		let fragment_shader_source_str = match unsafe { CStr::from_ptr(fragment_shader_source) }.to_str() {
				Ok(s) => s,
				Err(_) => return KgfxStatus::InvalidArg,
		};

	let result = std::panic::catch_unwind(|| unsafe {
		let shader_ptr = (*ctx).create_shader(vertex_shader_source_str, fragment_shader_source_str)?;
        *out_shader = shader_ptr;
        Ok::<(), KgfxStatus>(())
    });

    match result {
        Ok(Ok(())) => KgfxStatus::Ok,
        Ok(Err(s)) => s,
        Err(_) => KgfxStatus::Panic,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy_shader(_ctx: *mut GraphicsContext, shader: *mut KgfxShader) -> KgfxStatus {
	if shader.is_null() {
			return KgfxStatus::NullPointer;
	}

	let result = std::panic::catch_unwind(|| unsafe {
			// KgfxBuffer er en opaque handle; den peger i praksis på BufferInner.
			drop(Box::from_raw(shader as *mut ShaderInner));
	});

	match result {
			Ok(()) => KgfxStatus::Ok,
			Err(_) => KgfxStatus::Panic,
	}
}

// ==================== PIPELINE ====================

#[repr(C)]
pub struct KgfxPipeline { _private: [u8; 0] }

// Pipeline: i OpenGL ~ program + VAO layout; i Vulkan/DX ~ PSO/input layout
#[repr(C)]
pub struct KgfxPipelineDesc {
	pub shader: *mut KgfxShader,
	pub wireframe: bool,
}

pub(crate) enum PipelineBackend {
  OpenGL(OpenGLPipeline),
  // Vulkan(...), Dx11(...), Dx12(...)
}

pub(crate) struct PipelineInner {
  pub backend: PipelineBackend,
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_pipeline(ctx: *mut GraphicsContext, desc: KgfxPipelineDesc, out_pipeline: *mut *mut KgfxPipeline) -> KgfxStatus {
    if ctx.is_null() || out_pipeline.is_null() {
        return KgfxStatus::NullPointer;
    }

    unsafe { *out_pipeline = std::ptr::null_mut() };

    let result = std::panic::catch_unwind(|| unsafe {
        let pipeline_ptr = (*ctx).create_pipeline(desc)?;
        *out_pipeline = pipeline_ptr;
        Ok::<(), KgfxStatus>(())
    });

    match result {
        Ok(Ok(())) => KgfxStatus::Ok,
        Ok(Err(s)) => s,
        Err(_) => KgfxStatus::Panic,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy_pipeline(_ctx: *mut GraphicsContext, pipeline: *mut KgfxPipeline) -> KgfxStatus {
    if pipeline.is_null() {
        return KgfxStatus::NullPointer;
    }

    let result = std::panic::catch_unwind(|| unsafe {
        // KgfxBuffer er en opaque handle; den peger i praksis på BufferInner.
        drop(Box::from_raw(pipeline as *mut PipelineInner));
    });

    match result {
        Ok(()) => KgfxStatus::Ok,
        Err(_) => KgfxStatus::Panic,
    }
}


// ==================== BUFFER ====================
