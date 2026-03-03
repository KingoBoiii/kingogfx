use kingogfx::window::{Input, KeyCode, Window, WindowEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = Window::builder()
        .title("KingoGFX - Window Example (Rust)")
        .size(1280, 720)
        .build()?;

    window.focus();

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

        // Her ville du normalt render'e
        window.swap_buffers();
    }

    Ok(())
}