use kingogfx::window::{Input, KeyCode, Window, WindowEvent};
use kingogfx::graphics::{Graphics};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = Window::builder()
        .title("KingoGFX - Window Example (Rust)")
        .size(1280, 720)
        .build()?;

    window.focus();

    let graphics = Graphics::create(&mut window)?;
    graphics.clear_color(0.2, 0.3, 0.3, 1.0);

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

        graphics.clear();

        // Her ville du normalt render'e
        window.swap_buffers();
    }

    Ok(())
}