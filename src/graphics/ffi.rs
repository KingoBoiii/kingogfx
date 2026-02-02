use crate::graphics::backends::opengl::OpenGLPipeline;
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

// ==================== PIPELINE ====================

#[repr(C)]
pub struct KgfxPipeline { _private: [u8; 0] }

// Pipeline: i OpenGL ~ program + VAO layout; i Vulkan/DX ~ PSO/input layout
#[repr(C)]
pub struct KgfxPipelineDesc {
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
