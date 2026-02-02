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
pub enum GfxStatus {
    Ok = 0,
    NullPointer = 1,
    Unsupported = 2,
    InitFailed = 3,
    Panic = 255,
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_context(
    kind: BackendKind,
    window_handle: *mut WindowHandle,
    out_ctx: *mut *mut GraphicsContext,
) -> GfxStatus {
    if window_handle.is_null() || out_ctx.is_null() {
        return GfxStatus::NullPointer;
    }

    // Sæt altid out til null som default (så caller ikke får garbage ved fejl)
    unsafe { *out_ctx = std::ptr::null_mut() };

    let result = std::panic::catch_unwind(|| {
        let window = unsafe { &mut *window_handle };

        let backend = match kind {
            BackendKind::OpenGL => {
                let glb = backends::opengl::OpenGLBackend::new(window).ok_or(GfxStatus::InitFailed)?;
                Backend::OpenGL(glb)
            }
            _ => return Err(GfxStatus::Unsupported),
        };

        let ctx = GraphicsContext { backend };
        let ptr = Box::into_raw(Box::new(ctx));
        unsafe { *out_ctx = ptr };

        Ok::<(), GfxStatus>(())
    });

    match result {
        Ok(Ok(())) => GfxStatus::Ok,
        Ok(Err(status)) => status,
        Err(_) => GfxStatus::Panic,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy_context(ctx: *mut GraphicsContext) -> GfxStatus {
    if ctx.is_null() {
        return GfxStatus::NullPointer;
    }

    let result = std::panic::catch_unwind(|| unsafe {
        drop(Box::from_raw(ctx));
    });

    match result {
        Ok(()) => GfxStatus::Ok,
        Err(_) => GfxStatus::Panic,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear(ctx: *mut GraphicsContext) -> GfxStatus {
    if ctx.is_null() {
        return GfxStatus::NullPointer;
    }

    let result = std::panic::catch_unwind(|| unsafe {
        (*ctx).clear();
    });

    match result {
        Ok(()) => GfxStatus::Ok,
        Err(_) => GfxStatus::Panic,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear_color(
    ctx: *mut GraphicsContext,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) -> GfxStatus {
    if ctx.is_null() {
        return GfxStatus::NullPointer;
    }

    let result = std::panic::catch_unwind(|| unsafe {
        (*ctx).clear_color(red, green, blue, alpha);
    });

    match result {
        Ok(()) => GfxStatus::Ok,
        Err(_) => GfxStatus::Panic,
    }
}