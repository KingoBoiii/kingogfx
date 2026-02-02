use crate::graphics::{Backend, GraphicsContext};
use crate::graphics::backends;
use crate::{window::handle::WindowHandle};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum BackendKind {
    OpenGL = 0,
    Vulkan = 1,
    DirectX11 = 2,
    DirectX12 = 3,
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_create_context(kind: BackendKind, window_handle: *mut WindowHandle) -> *mut GraphicsContext {
  if window_handle.is_null() {
    return std::ptr::null_mut();
  }

    let result = std::panic::catch_unwind(|| {
        let window = unsafe { &mut *window_handle };

        let backend = match kind {
            BackendKind::OpenGL => {
                let glb = backends::opengl::OpenGLBackend::new(window)?;
                Backend::OpenGL(glb)
            }
            _ => return None, // unsupported for now
        };

        let ctx = GraphicsContext { backend };
        Some(Box::into_raw(Box::new(ctx)))
    });

    match result {
        Ok(Some(ptr)) => ptr,
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_destroy_context(ctx: *mut GraphicsContext) {
    if ctx.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| unsafe {
        drop(Box::from_raw(ctx));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear(ctx: *mut GraphicsContext) -> () {
  if ctx.is_null() {
    return;
  }

  let _ = std::panic::catch_unwind(|| unsafe {
      (*ctx).clear();
  });
}

#[unsafe(no_mangle)]
pub extern "C" fn kgfx_graphics_clear_color(ctx: *mut GraphicsContext, red: f32, green: f32, blue: f32, alpha: f32) -> () {
  let _ = std::panic::catch_unwind(|| unsafe {
      (*ctx).clear_color(red, green, blue, alpha);
  });
}