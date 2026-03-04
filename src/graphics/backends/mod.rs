pub(crate) mod opengl;
pub(crate) mod vulkan;

#[cfg(target_os = "windows")]
pub(crate) mod directx11;
#[cfg(target_os = "windows")]
pub(crate) mod directx12;

// Non-Windows builds: keep the crate compiling, but surface a runtime error if selected.
#[cfg(not(target_os = "windows"))]
pub(crate) mod directx11 {
	use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
	use crate::window::Window;
	use std::sync::Arc;

	pub(crate) struct DirectX11Graphics;
	pub(crate) struct DirectX11Buffer;
	pub(crate) struct DirectX11Pipeline;
	pub(crate) struct DirectX11Shader;

	impl DirectX11Graphics {
		pub(crate) fn create(_window: &mut Window) -> Result<Self, String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn set_viewport(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {}
		pub(crate) fn create_buffer_init(&mut self, _data: &[f32], _usage: BufferUsage) -> Result<DirectX11Buffer, String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn create_shader(&mut self, _desc: ShaderDescriptor<'_>) -> Result<Arc<DirectX11Shader>, String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn create_pipeline(&mut self, _shader: &Arc<DirectX11Shader>) -> Result<DirectX11Pipeline, String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn begin_frame(&mut self, _window: &mut Window, _clear: ClearColor) -> Result<(), String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn set_pipeline(&mut self, _pipeline: &DirectX11Pipeline) -> Result<(), String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn set_vertex_buffer(&mut self, _slot: u32, _buffer: &DirectX11Buffer) -> Result<(), String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn draw(&mut self, _vertex_count: u32, _first_vertex: u32) -> Result<(), String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn end_frame(&mut self, _window: &mut Window) -> Result<(), String> {
			Err("DirectX11 backend is only supported on Windows".to_string())
		}
		pub(crate) fn shutdown(&mut self, _window: &mut Window) -> Result<(), String> {
			Ok(())
		}
	}

	impl DirectX11Buffer {
		pub(crate) fn destroy(&mut self) {}
	}
	impl DirectX11Pipeline {
		pub(crate) fn destroy(&mut self) {}
	}
}

#[cfg(not(target_os = "windows"))]
pub(crate) mod directx12 {
	use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
	use crate::window::Window;
	use std::sync::Arc;

	pub(crate) struct DirectX12Graphics;
	pub(crate) struct DirectX12Buffer;
	pub(crate) struct DirectX12Pipeline;
	pub(crate) struct DirectX12Shader;

	impl DirectX12Graphics {
		pub(crate) fn create(_window: &mut Window) -> Result<Self, String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn set_viewport(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {}
		pub(crate) fn create_buffer_init(&mut self, _data: &[f32], _usage: BufferUsage) -> Result<DirectX12Buffer, String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn create_shader(&mut self, _desc: ShaderDescriptor<'_>) -> Result<Arc<DirectX12Shader>, String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn create_pipeline(&mut self, _shader: &Arc<DirectX12Shader>) -> Result<DirectX12Pipeline, String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn begin_frame(&mut self, _window: &mut Window, _clear: ClearColor) -> Result<(), String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn set_pipeline(&mut self, _pipeline: &DirectX12Pipeline) -> Result<(), String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn set_vertex_buffer(&mut self, _slot: u32, _buffer: &DirectX12Buffer) -> Result<(), String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn draw(&mut self, _vertex_count: u32, _first_vertex: u32) -> Result<(), String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn end_frame(&mut self, _window: &mut Window) -> Result<(), String> {
			Err("DirectX12 backend is only supported on Windows".to_string())
		}
		pub(crate) fn shutdown(&mut self, _window: &mut Window) -> Result<(), String> {
			Ok(())
		}
	}

	impl DirectX12Buffer {
		pub(crate) fn destroy(&mut self) {}
	}
	impl DirectX12Pipeline {
		pub(crate) fn destroy(&mut self) {}
	}
}