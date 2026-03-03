use kingogfx::window::{Input, KeyCode, Window, WindowEvent};
use kingogfx::graphics::{Graphics, GraphicsApi};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut window = Window::builder()
		.title("KingoGFX - Graphics Triangle Example (Rust)")
		.size(1280, 720)
		.build()?;

	window.focus();

	let graphics = Graphics::create(&mut window, GraphicsApi::OpenGL)?;
	graphics.viewport(0, 0, 1280, 720);
	graphics.clear_color(0.2, 0.3, 0.3, 1.0);

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

	let shader = graphics.create_shader(vs_src, fs_src)
		.expect("Failed to create shader");

	let vertices: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];

	let vertex_buffer = graphics.create_vertex_buffer(&vertices)
		.expect("Failed to create vertex buffer");

	let pipeline = graphics.create_pipeline()
		.expect("Failed to create pipeline");

	while !window.should_close() {
		for event in window.poll_events() {
			match event {
				WindowEvent::Close => {}
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

		graphics.clear();
		
		shader.bind();
		pipeline.bind();
		vertex_buffer.bind();
		graphics.draw_arrays(3);

		// Her ville du normalt render'e
		window.swap_buffers();
	}

	Ok(())
}