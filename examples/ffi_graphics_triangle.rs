use std::ffi::CString;

use kingogfx::kgfx_is_key_pressed;
use kingogfx::window::KgfxKeyCode;
use kingogfx::window::events::{KgfxEvent, KgfxEventKind};
use kingogfx::window::ffi::{
    KgfxWindow, KgfxWindowClientApi, kgfx_create_window, kgfx_destroy_window, kgfx_window_focus,
    kgfx_window_poll_event, kgfx_window_set_should_close, kgfx_window_should_close,
};
use kingogfx::graphics::ffi::{
    KgfxGraphics, KgfxPipeline, KgfxShader, KgfxVertexBuffer,
    KgfxStatus, KgfxApi, KgfxShaderLanguage, KgfxShaderCreateDesc,
    kgfx_graphics_create, kgfx_graphics_destroy, kgfx_graphics_shutdown,
    kgfx_graphics_viewport,
    kgfx_graphics_create_pipeline, kgfx_graphics_create_shader,
    kgfx_pipeline_destroy, kgfx_shader_destroy,
    kgfx_graphics_create_vertex_buffer, kgfx_vertex_buffer_destroy,
    kgfx_graphics_begin_frame, kgfx_graphics_end_frame,
    kgfx_graphics_set_pipeline, kgfx_graphics_set_vertex_buffer,
    kgfx_graphics_draw,
};

fn create_pipeline(graphics: *mut KgfxGraphics) -> (*mut KgfxShader, *mut KgfxPipeline) {
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
    let desc = KgfxShaderCreateDesc {
        vertex_language: KgfxShaderLanguage::Glsl,
        vertex_source: vs_c.as_ptr(),
        fragment_language: KgfxShaderLanguage::Glsl,
        fragment_source: fs_c.as_ptr(),
    };

    let status = kgfx_graphics_create_shader(graphics, &desc, &mut shader);
    if status != KgfxStatus::Ok || shader.is_null() {
        return (std::ptr::null_mut(), std::ptr::null_mut());
    }

    let mut pipeline: *mut KgfxPipeline = std::ptr::null_mut();
    let status = kgfx_graphics_create_pipeline(graphics, shader, &mut pipeline);
    if status == KgfxStatus::Ok { (shader, pipeline) } else { (shader, std::ptr::null_mut()) }
}

fn create_vertex_buffer(
    graphics: *mut KgfxGraphics,
    vertices: &[f32],
) -> *mut KgfxVertexBuffer {
    let mut buffer: *mut KgfxVertexBuffer = std::ptr::null_mut();
    let status = kgfx_graphics_create_vertex_buffer(
        graphics,
        vertices.as_ptr(),
        vertices.len(),
        &mut buffer,
    );

    if status == KgfxStatus::Ok { buffer } else { std::ptr::null_mut() }
}

fn main() {
    let title = CString::new("KingoGFX Triangle (FFI)").expect("title contains interior NUL");
    let window: *mut KgfxWindow =
        kgfx_create_window(title.as_ptr(), 1280, 720, KgfxWindowClientApi::NoApi);
    if window.is_null() {
        eprintln!("Failed to create window");
        return;
    }

    kgfx_window_focus(window);

    let mut graphics: *mut KgfxGraphics = std::ptr::null_mut();
    let status = kgfx_graphics_create(
        window as *mut _, // Window pointer as c_void
        KgfxApi::DirectX11,
        &mut graphics,
    );
    if status != KgfxStatus::Ok || graphics.is_null() {
        kgfx_destroy_window(window);
        eprintln!("Failed to create graphics");
        return;
    }

    kgfx_graphics_viewport(graphics, 0, 0, 1280, 720);

    let vertices: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];
    let vertex_buffer = create_vertex_buffer(graphics, &vertices);
    if vertex_buffer.is_null() {
        kgfx_graphics_destroy(graphics);
        kgfx_destroy_window(window);
        return;
    }

    let (shader, pipeline) = create_pipeline(graphics);
    if pipeline.is_null() {
        kgfx_vertex_buffer_destroy(vertex_buffer);
        kgfx_graphics_destroy(graphics);
        kgfx_destroy_window(window);
        return;
    }

    let mut event = KgfxEvent::default();

    while !kgfx_window_should_close(window) {
        while kgfx_window_poll_event(window, &mut event) {
            match event.kind {
                KgfxEventKind::Close => {
                    kgfx_window_set_should_close(window, true);
                }
                KgfxEventKind::Key => {
                    if kgfx_is_key_pressed(event, KgfxKeyCode::Escape) {
                        kgfx_window_set_should_close(window, true);
                    }
                }
                _ => {}
            }
        }

        if kgfx_window_should_close(window) {
            break;
        }

        if kgfx_graphics_begin_frame(graphics, window as *mut _, 0.2, 0.3, 0.3, 1.0) != KgfxStatus::Ok {
            continue;
        }

        if kgfx_graphics_set_pipeline(graphics, pipeline) != KgfxStatus::Ok {
            break;
        }
        if kgfx_graphics_set_vertex_buffer(graphics, 0, vertex_buffer) != KgfxStatus::Ok {
            break;
        }
        if kgfx_graphics_draw(graphics, 3, 0) != KgfxStatus::Ok {
            break;
        }
        if kgfx_graphics_end_frame(graphics, window as *mut _) != KgfxStatus::Ok {
            break;
        }
    }

    let _ = kgfx_graphics_shutdown(graphics, window as *mut _);

    kgfx_vertex_buffer_destroy(vertex_buffer);
    kgfx_pipeline_destroy(pipeline);
    kgfx_shader_destroy(shader);
    kgfx_graphics_destroy(graphics);
    kgfx_destroy_window(window);
}