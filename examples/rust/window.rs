use kingogfx::window::{KeyAction, Window, WindowEvent};

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
                WindowEvent::Key(key) => {
                    println!(
                        "Key event -> key: {}, scancode: {}, action: {:?}, mods: {}",
                        key.key, key.scancode, key.action, key.mods
                    );

                    // ESC (GLFW keycode 256) lukker vinduet ved key press
                    if key.key == 256 && key.action == KeyAction::Press {
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