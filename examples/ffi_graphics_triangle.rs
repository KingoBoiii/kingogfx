use std::ffi::{CString};

use kingogfx::kgfx_is_key_pressed;

use kingogfx::window::KgfxKeyCode;
use kingogfx::window::events::{KgfxEvent, KgfxEventKind};
use kingogfx::window::ffi::{
    KgfxWindow, kgfx_create_window, kgfx_destroy_window, kgfx_window_focus, kgfx_window_poll_event, kgfx_window_set_should_close, kgfx_window_should_close, kgfx_window_swap_buffers
};

use kingogfx::graphics::GraphicsContext;
use kingogfx::graphics::ffi::{
    kgfx_graphics_clear, kgfx_graphics_clear_color, kgfx_graphics_create_buffer,
    kgfx_graphics_create_context, kgfx_graphics_create_pipeline, kgfx_graphics_create_shader,
    kgfx_graphics_destroy_buffer, kgfx_graphics_destroy_context, kgfx_graphics_destroy_pipeline,
    kgfx_graphics_destroy_shader, kgfx_graphics_draw_arrays, kgfx_graphics_viewport, BackendKind,
    KgfxBuffer, KgfxBufferDesc, KgfxBufferUsage, KgfxPipeline, KgfxPipelineDesc, KgfxShader,
    KgfxStatus,
};

fn create_shader(ctx: *mut GraphicsContext) -> *mut KgfxShader {
    let vs_src = r#"
        #version 330 core
        layout (location = 0) in vec2 aPos;
        void main() {
            gl_Position = vec4(aPos.xy, 0.0, 1.0);
        }
    "#;

    let fs_src = r#"
        #version 330 core
        out vec4 FragColor;
        void main() {
            FragColor = vec4(1.0, 0.6, 0.2, 1.0);
        }
    "#;

    let vs_c = CString::new(vs_src).expect("vertex shader contains interior NUL");
    let fs_c = CString::new(fs_src).expect("fragment shader contains interior NUL");

    let mut shader: *mut KgfxShader = std::ptr::null_mut();
    let status = kgfx_graphics_create_shader(ctx, vs_c.as_ptr(), fs_c.as_ptr(), &mut shader);

    if status == KgfxStatus::Ok { shader } else { std::ptr::null_mut() }
}

fn create_pipeline(ctx: *mut GraphicsContext, shader: *mut KgfxShader) -> *mut KgfxPipeline {
    let desc = KgfxPipelineDesc {
        shader,
        wireframe: false,
    };

    let mut pipeline: *mut KgfxPipeline = std::ptr::null_mut();
    let status = kgfx_graphics_create_pipeline(ctx, desc, &mut pipeline);

    if status == KgfxStatus::Ok { pipeline } else { std::ptr::null_mut() }
}

fn create_vertex_buffer(
    ctx: *mut GraphicsContext,
    initial_data: *const u8,
    size: usize,
) -> *mut KgfxBuffer {
    let desc = KgfxBufferDesc {
        struct_size: std::mem::size_of::<KgfxBufferDesc>() as u32,
        usage: KgfxBufferUsage::Vertex,
        size_bytes: size,
    };

    let mut buffer: *mut KgfxBuffer = std::ptr::null_mut();
    let status = kgfx_graphics_create_buffer(ctx, desc, initial_data, &mut buffer);

    if status == KgfxStatus::Ok { buffer } else { std::ptr::null_mut() }
}

fn main() {
    let title = CString::new("KingoGFX Triangle (FFI)").expect("title contains interior NUL");
    let window: *mut KgfxWindow = kgfx_create_window(title.as_ptr(), 1280, 720);
    if window.is_null() {
        eprintln!("Failed to create window");
        return;
    }
    
    kgfx_window_focus(window);

    let mut ctx: *mut GraphicsContext = std::ptr::null_mut();
    let status = kgfx_graphics_create_context(BackendKind::OpenGL, window, &mut ctx);
    if status != KgfxStatus::Ok || ctx.is_null() {
        kgfx_destroy_window(window);
        eprintln!("Failed to create graphics context");
        return;
    }

    kgfx_graphics_viewport(ctx, 0, 0, 1280, 720);

    let shader = create_shader(ctx);
    if shader.is_null() {
        kgfx_graphics_destroy_context(ctx);
        kgfx_destroy_window(window);
        return;
    }

    let pipeline = create_pipeline(ctx, shader);
    if pipeline.is_null() {
        kgfx_graphics_destroy_shader(ctx, shader);
        kgfx_graphics_destroy_context(ctx);
        kgfx_destroy_window(window);
        return;
    }

    let vertices: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];
    let vertex_buffer = create_vertex_buffer(
        ctx,
        vertices.as_ptr() as *const u8,
        vertices.len() * std::mem::size_of::<f32>(),
    );
    if vertex_buffer.is_null() {
        kgfx_graphics_destroy_pipeline(ctx, pipeline);
        kgfx_graphics_destroy_shader(ctx, shader);
        kgfx_graphics_destroy_context(ctx);
        kgfx_destroy_window(window);
        return;
    }

    let mut event = KgfxEvent::default();

    while !kgfx_window_should_close(window) {
        kgfx_graphics_clear_color(ctx, 0.1, 0.2, 0.3, 1.0);
        kgfx_graphics_clear(ctx);
        kgfx_graphics_draw_arrays(ctx, pipeline, 3);

        while kgfx_window_poll_event(window, &mut event) {
                if let KgfxEventKind::Key = event.kind {
                if let Some(k) = event.as_key() {
                    if kgfx_is_key_pressed(k, KgfxKeyCode::Escape) {
                        kgfx_window_set_should_close(window, true);
                    }
                }
            }
        }

        kgfx_window_swap_buffers(window);
    }

    kgfx_graphics_destroy_buffer(ctx, vertex_buffer);
    kgfx_graphics_destroy_pipeline(ctx, pipeline);
    kgfx_graphics_destroy_shader(ctx, shader);
    kgfx_graphics_destroy_context(ctx);
    kgfx_destroy_window(window);
}