use kingogfx::graphics::shader::Shader;
use kingogfx::window::{Input, KeyCode, Window, WindowEvent};
use kingogfx::graphics::{Graphics, GraphicsApi};

fn create_shader(gfx: &Graphics) -> Shader {
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

	gfx.create_shader(vs_src, fs_src)
		.expect("Failed to create shader")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut window = Window::builder()
		.title("KingoGFX - Graphics Triangle Example (Rust)")
		.size(1280, 720)
		.build()?;

	window.focus();

	let graphics = Graphics::create(&mut window, GraphicsApi::OpenGL)?;
	graphics.viewport(0, 0, 1280, 720);
	graphics.clear_color(0.2, 0.3, 0.3, 1.0);

	let shader = create_shader(&graphics);

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

		// Her ville du normalt render'e
		window.swap_buffers();
	}

	Ok(())
}