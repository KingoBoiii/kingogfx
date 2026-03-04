use kingogfx::window::builder::WindowClientApi;
use kingogfx::window::{Input, KeyCode, Window, WindowEvent};
use kingogfx::graphics::{BufferUsage, ClearColor, Graphics, GraphicsApi, PipelineDescriptor, ShaderDescriptor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut window = Window::builder()
		.title("KingoGFX - Graphics Triangle Example (Rust)")
		.size(1280, 720)
		.client_api(WindowClientApi::NoApi)
		.build()?;

	window.focus();

	let graphics = Graphics::create(&mut window, GraphicsApi::Vulkan)?;
	let mut graphics = graphics;
	graphics.set_viewport(0, 0, 1280, 720);

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

	let shader = graphics
		.create_shader(ShaderDescriptor {
			vertex_source_glsl: vs_src,
			fragment_source_glsl: fs_src,
		})
		.expect("Failed to create shader");

	let pipeline = graphics
		.create_pipeline(PipelineDescriptor { shader: &shader })
		.expect("Failed to create render pipeline");

	let vertices: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];

	let vertex_buffer = graphics
		.create_buffer_init(&vertices, BufferUsage::Vertex)
		.expect("Failed to create vertex buffer");

	while !window.should_close() {
		for event in window.poll_events() {
			match event {
				WindowEvent::Close => {
					window.set_should_close(true);
				}
				WindowEvent::Key(key_event) => {
					println!(
						"Key event -> key: {:?}, action: {:?}, mods: {:?}",
						key_event.key_code, key_event.action, key_event.modifiers
					);

					if Input::is_key_pressed(key_event, KeyCode::Escape) {
						window.set_should_close(true);
					}
				}
				WindowEvent::Unknown => {}
			}
		}

		if window.should_close() {
			break;
		}

		if graphics
			.begin_frame(&mut window, ClearColor { r: 0.2, g: 0.3, b: 0.3, a: 1.0 })
			.is_err()
		{
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