use std::ffi::CString;

use kingogfx::graphics::buffer::{KgfxBuffer, KgfxBufferDesc, KgfxBufferUsage, kgfx_graphics_create_buffer, kgfx_graphics_destroy_buffer};
use kingogfx::window::{
  KgfxEvent, KgfxEventKind,
  kgfx_create_window, kgfx_destroy_window, kgfx_window_poll_event,
  kgfx_window_set_should_close, kgfx_window_should_close, kgfx_window_swap_buffers,
};
use kingogfx::graphics::{
  GraphicsContext, KgfxPipeline, KgfxPipelineDesc, KgfxStatus, kgfx_graphics_clear, kgfx_graphics_clear_color, kgfx_graphics_create_context, kgfx_graphics_create_pipeline, kgfx_graphics_destroy_pipeline, kgfx_graphics_viewport
};
use kingogfx::{kgfx_is_key_pressed};

fn create_pipeline(ctx: *mut GraphicsContext) -> *mut KgfxPipeline {
  let desc = KgfxPipelineDesc {
    wireframe: false
  };

  let mut pipeline: *mut KgfxPipeline = std::ptr::null_mut();

  let status = kgfx_graphics_create_pipeline(ctx, desc, &mut pipeline);
  if status != KgfxStatus::Ok || pipeline.is_null() {
    return std::ptr::null_mut();
  }

  return pipeline;
}

fn create_vertex_buffer(ctx: *mut GraphicsContext, initial_data: *const u8, size: usize) -> *mut KgfxBuffer {
  let mut buffer: *mut KgfxBuffer = std::ptr::null_mut();

  let desc = KgfxBufferDesc {
    struct_size: std::mem::size_of::<KgfxBufferDesc>() as u32,
    usage: KgfxBufferUsage::Vertex,
    size_bytes: size,
  };

  let status = kgfx_graphics_create_buffer(ctx, desc, initial_data, &mut buffer);
  if status != KgfxStatus::Ok || buffer.is_null() {
    return std::ptr::null_mut();
  }

  return buffer;
}

fn main() {
  let title = CString::new("GLFW window").expect("title contains an interior NUL byte");
  let handle = kgfx_create_window(title.as_ptr(), 800, 600);

  let mut ctx: *mut GraphicsContext = std::ptr::null_mut();
  let status = kgfx_graphics_create_context(kingogfx::graphics::BackendKind::OpenGL, handle, &mut ctx);
  if status != KgfxStatus::Ok
  {
    return;
  }

  kgfx_graphics_viewport(ctx, 0, 0, 800, 600);

  let pipeline: *mut KgfxPipeline = create_pipeline(ctx);
  if pipeline.is_null() {
    kgfx_destroy_window(handle);
    return;
  }

  let vertices: [f32; 6] = [
    -0.5, -0.5,
      0.5, -0.5,
      0.0,  0.5,
  ];

  let initial_data = vertices.as_ptr() as *const u8;
  let vertex_buffer: *mut KgfxBuffer = create_vertex_buffer(ctx, initial_data, vertices.len() * std::mem::size_of::<f32>());
  if vertex_buffer.is_null() {
    kgfx_destroy_window(handle);
    return;
  }

  let mut event = KgfxEvent::default();

  while !kgfx_window_should_close(handle) {
    kgfx_graphics_clear_color(ctx, 0.1, 0.2, 0.3, 1.0);
    kgfx_graphics_clear(ctx);

    while kgfx_window_poll_event(handle, &mut event) {
      match event.kind {
        KgfxEventKind::Key => {
          if let Some(k) = event.as_key() {
            // key 256 = escape key
            if kgfx_is_key_pressed(k, 256) {
              kgfx_window_set_should_close(handle, true);
            }
          }
        }

        _ => {}
      }
    }

    kgfx_window_swap_buffers(handle);
  }

  kgfx_graphics_destroy_buffer(ctx, vertex_buffer);
  kgfx_graphics_destroy_pipeline(ctx, pipeline);

  kgfx_destroy_window(handle);
}