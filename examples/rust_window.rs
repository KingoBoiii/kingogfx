use kingogfx::{window::{Key, KeyAction, Window, WindowEvent}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = Window::builder()
        .title("KingoGFX - Window Example (Rust)")
        .size(1280, 720)
        .build()?;

    window.focus();

    while !window.should_close() {
        for event in window.poll_events() {
            match event {
                WindowEvent::Close => {
                    window.set_should_close(true);
                }
                WindowEvent::Key(key_event) => {
                    println!(
                        "Key event -> key: {:?}, action: {:?}, mods: {:?}",
                        key_event.key, key_event.action, key_event.modifiers
                    );

                    // ESC (GLFW keycode 256) lukker vinduet ved key press
                    if key_event.key == Key::Escape && key_event.action == KeyAction::Press {
                        window.set_should_close(true);
                    }
                }
                WindowEvent::Unknown => {}
            }
        }

        // Her ville du normalt render'e
        window.swap_buffers();
    }

    Ok(())
}