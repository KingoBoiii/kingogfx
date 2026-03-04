# KingoGFX

Minimal graphics + window library with multiple backends:

- Graphics: Vulkan, OpenGL, DirectX 11, DirectX 12
- Window/events: GLFW backend
- Shaders: accepts **GLSL or HLSL** as input; internally compiles to **SPIR-V** (and cross-compiles as needed per backend)

This repo builds as a normal Rust crate and also exposes a C ABI (FFI) via a generated header.

## Build / Run (Rust)

```bash
cargo check

# Rust examples
cargo run --example rust_window
cargo run --example rust_graphics_triangle
cargo run --example rust_graphics_triangle_hlsl

# FFI examples (still written in Rust, but calling the C ABI)
cargo run --example ffi_window
cargo run --example ffi_graphics_triangle
cargo run --example ffi_graphics_triangle_hlsl
```

## Rust usage (native API)

A minimal “triangle-ish” skeleton using the safe Rust API:

```rust
use kingogfx::graphics::{
    BufferUsage, ClearColor, Graphics, GraphicsApi, PipelineDescriptor,
    ShaderDescriptor, ShaderSource,
};
use kingogfx::window::builder::{WindowClientApi};
use kingogfx::window::{Window, WindowEvent};

fn main() {
    // For Vulkan/DX backends you typically want a window without an OpenGL context.
    // IMPORTANT: If you use the OpenGL backend, you MUST set `.client_api(WindowClientApi::OpenGl)`.
    let mut window = Window::builder()
        .title("KingoGFX (Rust)")
        .size(1280, 720)
        .client_api(WindowClientApi::NoApi)
        .build()
        .expect("window");

    let mut gfx = Graphics::create(&mut window, GraphicsApi::Vulkan).expect("graphics");
    gfx.set_viewport(0, 0, 1280, 720);

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

    // You can swap to ShaderSource::hlsl("...") as well.
    let shader = gfx
        .create_shader(ShaderDescriptor {
            vertex: ShaderSource::glsl(vs_src),
            fragment: ShaderSource::glsl(fs_src),
        })
        .expect("shader");

    let pipeline = gfx
        .create_pipeline(PipelineDescriptor { shader: &shader })
        .expect("pipeline");

    let vertices: [f32; 6] = [-0.5, -0.5, 0.5, -0.5, 0.0, 0.5];
    let vb = gfx
        .create_buffer_init(&vertices, BufferUsage::Vertex)
        .expect("vertex buffer");

    while !window.should_close() {
        while let Some(ev) = window.poll_event() {
            match ev {
                WindowEvent::Close => window.set_should_close(true),
                _ => {}
            }
        }

        gfx.begin_frame(&mut window, ClearColor { r: 0.2, g: 0.3, b: 0.3, a: 1.0 })
            .expect("begin_frame");
        gfx.set_pipeline(&pipeline).expect("set_pipeline");
        gfx.set_vertex_buffer(0, &vb).expect("set_vertex_buffer");
        gfx.draw(3, 0).expect("draw");
        gfx.end_frame(&mut window).expect("end_frame");
    }
}
```

## C / C++ (FFI)

### 1) Build the library

On Windows, a release build will typically produce outputs under `target\\release\\`:

```bash
cargo build --release
```

This crate is configured as `cdylib` and `staticlib` (see `Cargo.toml`). Using the DLL is usually the simplest from C++.

### 2) Generate the header (cbindgen)

This repo includes `cbindgen.toml`. Install and generate a header:

```bash
cargo install cbindgen
mkdir include
cbindgen --config cbindgen.toml --crate kingogfx --output include/kingogfx.h
```

`cbindgen.toml` sets `namespace = "kgfx"`, so the header symbols will be inside `namespace kgfx { ... }` in C++.

### 3) Minimal C++ example (using the C ABI)

`main.cpp`:

```cpp
#include <cstdint>
#include <cstdio>

#include "kingogfx.h"

int main() {
    using namespace kgfx;

    KgfxWindow* window = kgfx_create_window("KingoGFX Triangle (C++)", 1280, 720, KgfxWindowClientApi::NoApi);
    if (!window) {
        std::fprintf(stderr, "Failed to create window\n");
        return 1;
    }

    kgfx_window_focus(window);

    KgfxGraphics* graphics = nullptr;
    if (kgfx_graphics_create((void*)window, KgfxApi::Vulkan, &graphics) != KgfxStatus::Ok || !graphics) {
        std::fprintf(stderr, "Failed to create graphics\n");
        kgfx_destroy_window(window);
        return 1;
    }

    kgfx_graphics_viewport(graphics, 0, 0, 1280, 720);

    // Vertex buffer (vec2 positions)
    float vertices[] = { -0.5f, -0.5f,  0.5f, -0.5f,  0.0f,  0.5f };

    KgfxVertexBuffer* vb = nullptr;
    if (kgfx_graphics_create_vertex_buffer(graphics, vertices, (size_t)(sizeof(vertices) / sizeof(float)), &vb) != KgfxStatus::Ok || !vb) {
        std::fprintf(stderr, "Failed to create vertex buffer\n");
        kgfx_graphics_destroy(graphics);
        kgfx_destroy_window(window);
        return 1;
    }

    // Shaders (GLSL input). You can set language to Hlsl as well.
    const char* vs_src = R"(
        #version 330 core
        layout (location = 0) in vec2 aPos;
        void main() {
            gl_Position = vec4(aPos.xy, 0.0, 1.0);
        }
    )";

    const char* fs_src = R"(
        #version 330 core
        out vec4 FragColor;
        void main() {
            FragColor = vec4(1.0, 0.6, 0.2, 1.0);
        }
    )";

    KgfxShader* shader = nullptr;
    KgfxShaderCreateDesc shader_desc{};
    shader_desc.vertex_language = KgfxShaderLanguage::Glsl;
    shader_desc.vertex_source = vs_src;
    shader_desc.fragment_language = KgfxShaderLanguage::Glsl;
    shader_desc.fragment_source = fs_src;

    if (kgfx_graphics_create_shader(graphics, &shader_desc, &shader) != KgfxStatus::Ok || !shader) {
        std::fprintf(stderr, "Failed to create shader\n");
        kgfx_vertex_buffer_destroy(vb);
        kgfx_graphics_destroy(graphics);
        kgfx_destroy_window(window);
        return 1;
    }

    KgfxPipeline* pipeline = nullptr;
    if (kgfx_graphics_create_pipeline(graphics, shader, &pipeline) != KgfxStatus::Ok || !pipeline) {
        std::fprintf(stderr, "Failed to create pipeline\n");
        kgfx_shader_destroy(shader);
        kgfx_vertex_buffer_destroy(vb);
        kgfx_graphics_destroy(graphics);
        kgfx_destroy_window(window);
        return 1;
    }

    KgfxEvent ev{};

    while (!kgfx_window_should_close(window)) {
        while (kgfx_window_poll_event(window, &ev)) {
            if (ev.kind == KgfxEventKind::Close) {
                kgfx_window_set_should_close(window, true);
            }
        }

        if (kgfx_graphics_begin_frame(graphics, (void*)window, 0.2f, 0.3f, 0.3f, 1.0f) != KgfxStatus::Ok) {
            continue;
        }

        if (kgfx_graphics_set_pipeline(graphics, pipeline) != KgfxStatus::Ok) break;
        if (kgfx_graphics_set_vertex_buffer(graphics, 0, vb) != KgfxStatus::Ok) break;
        if (kgfx_graphics_draw(graphics, 3, 0) != KgfxStatus::Ok) break;
        if (kgfx_graphics_end_frame(graphics, (void*)window) != KgfxStatus::Ok) break;
    }

    (void)kgfx_graphics_shutdown(graphics, (void*)window);

    kgfx_pipeline_destroy(pipeline);
    kgfx_shader_destroy(shader);
    kgfx_vertex_buffer_destroy(vb);
    kgfx_graphics_destroy(graphics);
    kgfx_destroy_window(window);

    return 0;
}
```

Notes:
- The example uses `KgfxApi::Vulkan`. You can switch to `DirectX12` / `DirectX11` on Windows.
- For OpenGL, you MUST create the window with `KgfxWindowClientApi::OpenGl` (so you get an OpenGL context).

