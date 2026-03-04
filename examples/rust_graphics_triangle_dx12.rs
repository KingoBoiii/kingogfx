use kingogfx::window::builder::WindowClientApi;
use kingogfx::window::{Input, KeyCode, Window, WindowEvent};
use kingogfx::graphics::{BufferUsage, ClearColor, Graphics, GraphicsApi, PipelineDescriptor, ShaderDescriptor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = Window::builder()
        .title("KingoGFX - Graphics Triangle Example (DirectX12)")
        .size(1280, 720)
        .client_api(WindowClientApi::NoApi)
        .build()?;

    window.focus();

    let mut graphics = Graphics::create(&mut window, GraphicsApi::DirectX12)?;
    graphics.set_viewport(0, 0, 1280, 720);

    let vs_hlsl = r#"
        struct VSIn {
            float2 pos : POSITION;
        };

        struct VSOut {
            float4 pos : SV_POSITION;
        };

        VSOut main(VSIn input) {
            VSOut o;
            o.pos = float4(input.pos, 0.0, 1.0);
            return o;
        }
    "#;

    let ps_hlsl = r#"
        float4 main() : SV_TARGET {
            return float4(1.0, 0.6, 0.2, 1.0);
        }
    "#;

    let shader = graphics.create_shader(ShaderDescriptor {
        vertex_source_glsl: "",
        fragment_source_glsl: "",
        vertex_source_hlsl: Some(vs_hlsl),
        fragment_source_hlsl: Some(ps_hlsl),
    })?;

    let pipeline = graphics.create_pipeline(PipelineDescriptor { shader: &shader })?;

    let vertices: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];
    let vertex_buffer = graphics.create_buffer_init(&vertices, BufferUsage::Vertex)?;

    while !window.should_close() {
        for event in window.poll_events() {
            match event {
                WindowEvent::Close => window.set_should_close(true),
                WindowEvent::Key(key_event) => {
                    if Input::is_key_pressed(key_event, KeyCode::Escape) {
                        window.set_should_close(true);
                    }
                }
                WindowEvent::Unknown => {}
            }
        }

        if graphics.begin_frame(&mut window, ClearColor { r: 0.2, g: 0.3, b: 0.3, a: 1.0 }).is_err() {
            continue;
        }

        graphics.set_pipeline(&pipeline)?;
        graphics.set_vertex_buffer(0, &vertex_buffer)?;
        graphics.draw(3, 0)?;
        graphics.end_frame(&mut window)?;
    }

    let _ = graphics.shutdown(&mut window);
    Ok(())
}
