use std::ffi::CString;
use std::sync::Arc;

use glfw_sys::GLFWwindow;

use windows::Win32::Foundation::*;
use windows::core::PCSTR;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Direct3D::Fxc::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
use crate::window::Window;

use super::directx11_swapchain::DirectX11Swapchain;
use super::{DirectX11Buffer, DirectX11Pipeline, DirectX11Shader};

pub(crate) struct DirectX11Graphics {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swapchain: DirectX11Swapchain,

    viewport: Option<D3D11_VIEWPORT>,

    in_frame: bool,
}

impl DirectX11Graphics {
    pub(crate) fn create(window: &mut Window) -> Result<Self, String> {
        let hwnd = hwnd_from_window(window)?;
        let (width, height) = framebuffer_size_u32(window);

        unsafe {
            let swap_desc = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC {
                    Width: width,
                    Height: height,
                    RefreshRate: DXGI_RATIONAL { Numerator: 60, Denominator: 1 },
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                    Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
                },
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                OutputWindow: hwnd,
                Windowed: BOOL(1),
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Flags: 0,
            };

            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            let mut swapchain: Option<IDXGISwapChain> = None;

            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&swap_desc as *const _),
                Some(&mut swapchain as *mut _),
                Some(&mut device as *mut _),
                None,
                Some(&mut context as *mut _),
            )
            .map_err(|e| format!("D3D11CreateDeviceAndSwapChain failed: {e:?}"))?;

            let device = device.ok_or_else(|| "D3D11 device is null".to_string())?;
            let context = context.ok_or_else(|| "D3D11 context is null".to_string())?;
            let swapchain = swapchain.ok_or_else(|| "DXGI swapchain is null".to_string())?;

            let rtv = create_rtv(&device, &swapchain)?;

            Ok(Self {
                device,
                context,
                swapchain: DirectX11Swapchain {
                    swapchain,
                    rtv,
                    width,
                    height,
                },
                viewport: None,
                in_frame: false,
            })
        }
    }

    pub(crate) fn shutdown(&mut self, _window: &mut Window) -> Result<(), String> {
        Ok(())
    }

    pub(crate) fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.viewport = Some(D3D11_VIEWPORT {
            TopLeftX: x as f32,
            TopLeftY: y as f32,
            Width: width.max(0) as f32,
            Height: height.max(0) as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        });
    }

    pub(crate) fn create_buffer_init(&mut self, data: &[f32], usage: BufferUsage) -> Result<DirectX11Buffer, String> {
        if data.is_empty() {
            return Err("buffer data is empty".to_string());
        }
        match usage {
            BufferUsage::Vertex => {}
        }

        let size_bytes = (data.len() * std::mem::size_of::<f32>()) as u32;
        let stride_bytes = (2 * std::mem::size_of::<f32>()) as u32;

        unsafe {
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: size_bytes,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            let init = D3D11_SUBRESOURCE_DATA {
                pSysMem: data.as_ptr() as *const _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };

            let mut buffer: Option<ID3D11Buffer> = None;
            self.device
                .CreateBuffer(&desc, Some(&init), Some(&mut buffer as *mut _))
                .map_err(|e| format!("ID3D11Device::CreateBuffer failed: {e:?}"))?;
            let buffer = buffer.ok_or_else(|| "CreateBuffer returned null buffer".to_string())?;

            Ok(DirectX11Buffer {
                buffer: Some(buffer),
                stride_bytes,
            })
        }
    }

    pub(crate) fn create_shader(&mut self, desc: ShaderDescriptor<'_>) -> Result<Arc<DirectX11Shader>, String> {
        let vs_src = desc
            .vertex_source_hlsl
            .ok_or_else(|| "DirectX11 shader creation requires vertex_source_hlsl".to_string())?;
        let ps_src = desc
            .fragment_source_hlsl
            .ok_or_else(|| "DirectX11 shader creation requires fragment_source_hlsl".to_string())?;

        let vs = compile_hlsl(vs_src, "main", "vs_5_0")?;
        let ps = compile_hlsl(ps_src, "main", "ps_5_0")?;
        Ok(Arc::new(DirectX11Shader { vs, ps }))
    }

    pub(crate) fn create_pipeline(&mut self, shader: &Arc<DirectX11Shader>) -> Result<DirectX11Pipeline, String> {
        unsafe {
            let vs_bytes = std::slice::from_raw_parts(shader.vs.GetBufferPointer() as *const u8, shader.vs.GetBufferSize());
            let ps_bytes = std::slice::from_raw_parts(shader.ps.GetBufferPointer() as *const u8, shader.ps.GetBufferSize());

            let mut vs: Option<ID3D11VertexShader> = None;
            self.device
                .CreateVertexShader(vs_bytes, None, Some(&mut vs as *mut _))
                .map_err(|e| format!("CreateVertexShader failed: {e:?}"))?;
            let vs = vs.ok_or_else(|| "CreateVertexShader returned null shader".to_string())?;

            let mut ps: Option<ID3D11PixelShader> = None;
            self.device
                .CreatePixelShader(ps_bytes, None, Some(&mut ps as *mut _))
                .map_err(|e| format!("CreatePixelShader failed: {e:?}"))?;
            let ps = ps.ok_or_else(|| "CreatePixelShader returned null shader".to_string())?;

            static SEMANTIC_POSITION: &[u8] = b"POSITION\0";
            let element = D3D11_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(SEMANTIC_POSITION.as_ptr()),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            };

            let mut layout: Option<ID3D11InputLayout> = None;
            self.device
                .CreateInputLayout(
                    &[element],
                    vs_bytes,
                    Some(&mut layout as *mut _),
                )
                .map_err(|e| format!("CreateInputLayout failed: {e:?}"))?;
            let layout = layout.ok_or_else(|| "CreateInputLayout returned null layout".to_string())?;

            let rast_desc = D3D11_RASTERIZER_DESC {
                FillMode: D3D11_FILL_SOLID,
                CullMode: D3D11_CULL_NONE,
                FrontCounterClockwise: BOOL(0),
                DepthBias: 0,
                DepthBiasClamp: 0.0,
                SlopeScaledDepthBias: 0.0,
                DepthClipEnable: BOOL(1),
                ScissorEnable: BOOL(0),
                MultisampleEnable: BOOL(0),
                AntialiasedLineEnable: BOOL(0),
            };
            let mut raster_state: Option<ID3D11RasterizerState> = None;
            self.device
                .CreateRasterizerState(&rast_desc, Some(&mut raster_state as *mut _))
                .map_err(|e| format!("CreateRasterizerState failed: {e:?}"))?;
            let raster_state = raster_state.ok_or_else(|| "CreateRasterizerState returned null state".to_string())?;

            Ok(DirectX11Pipeline {
                vs,
                ps,
                input_layout: layout,
                raster_state,
            })
        }
    }

    pub(crate) fn begin_frame(&mut self, window: &mut Window, clear: ClearColor) -> Result<(), String> {
        if self.in_frame {
            return Err("begin_frame called while already in a frame".to_string());
        }

        self.resize_if_needed(window)?;

        unsafe {
            self.context.OMSetRenderTargets(Some(&[Some(self.swapchain.rtv.clone())]), None);

            let color = [clear.r, clear.g, clear.b, clear.a];
            self.context.ClearRenderTargetView(&self.swapchain.rtv, &color);

            let vp = self.viewport.unwrap_or(D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: self.swapchain.width as f32,
                Height: self.swapchain.height as f32,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            });
            self.context.RSSetViewports(Some(&[vp]));
        }

        self.in_frame = true;
        Ok(())
    }

    pub(crate) fn set_pipeline(&mut self, pipeline: &DirectX11Pipeline) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_pipeline must be called between begin_frame/end_frame".to_string());
        }

        unsafe {
            self.context.IASetInputLayout(&pipeline.input_layout);
            self.context.VSSetShader(&pipeline.vs, None);
            self.context.PSSetShader(&pipeline.ps, None);
            self.context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.context.RSSetState(Some(&pipeline.raster_state));
        }

        Ok(())
    }

    pub(crate) fn set_vertex_buffer(&mut self, slot: u32, buffer: &DirectX11Buffer) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_vertex_buffer must be called between begin_frame/end_frame".to_string());
        }
        if slot != 0 {
            return Err("DirectX11 backend currently supports only slot 0".to_string());
        }
        let vb = buffer
            .buffer
            .as_ref()
            .ok_or_else(|| "DirectX11 buffer is null".to_string())?;

        unsafe {
            let stride = buffer.stride_bytes;
            let offset = 0u32;
            let buffers = [Some(vb.clone())];
            let strides = [stride];
            let offsets = [offset];
            self.context.IASetVertexBuffers(0, 1, Some(buffers.as_ptr()), Some(strides.as_ptr()), Some(offsets.as_ptr()));
        }

        Ok(())
    }

    pub(crate) fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), String> {
        if !self.in_frame {
            return Err("draw must be called between begin_frame/end_frame".to_string());
        }

        unsafe {
            self.context.Draw(vertex_count, first_vertex);
        }
        Ok(())
    }

    pub(crate) fn end_frame(&mut self, _window: &mut Window) -> Result<(), String> {
        if !self.in_frame {
            return Err("end_frame called without begin_frame".to_string());
        }

        unsafe {
            self.swapchain
                .swapchain
                .Present(1, DXGI_PRESENT(0))
                .ok()
                .map_err(|e| format!("IDXGISwapChain::Present failed: {e:?}"))?;
        }

        self.in_frame = false;
        Ok(())
    }

    fn resize_if_needed(&mut self, window: &mut Window) -> Result<(), String> {
        let (w, h) = framebuffer_size_u32(window);
        if w == 0 || h == 0 {
            return Err("Framebuffer size is 0 (minimized)".to_string());
        }

        if w == self.swapchain.width && h == self.swapchain.height {
            return Ok(());
        }

        unsafe {
            // Unbind RT before resizing.
            self.context.OMSetRenderTargets(None, None);

            self.swapchain
                .swapchain
                .ResizeBuffers(0, w, h, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0))
                .map_err(|e| format!("IDXGISwapChain::ResizeBuffers failed: {e:?}"))?;

            self.swapchain.rtv = create_rtv(&self.device, &self.swapchain.swapchain)?;
            self.swapchain.width = w;
            self.swapchain.height = h;
        }

        Ok(())
    }
}

fn framebuffer_size_u32(window: &Window) -> (u32, u32) {
    let (w, h) = window.framebuffer_size();
    (w.max(0) as u32, h.max(0) as u32)
}

fn hwnd_from_window(window: &mut Window) -> Result<HWND, String> {
    let glfw_window = window.glfw_window_ptr();
    if glfw_window.is_null() {
        return Err("Window backend did not provide a valid GLFWwindow pointer".to_string());
    }

    unsafe {
        let hwnd = glfw_sys::glfwGetWin32Window(glfw_window as *mut GLFWwindow);
        if hwnd.is_null() {
            return Err("glfwGetWin32Window returned null HWND".to_string());
        }
        Ok(HWND(hwnd))
    }
}

fn compile_hlsl(source: &str, entry: &str, target: &str) -> Result<ID3DBlob, String> {
    unsafe {
        let entry = CString::new(entry).map_err(|_| "entry contains NUL".to_string())?;
        let target = CString::new(target).map_err(|_| "target contains NUL".to_string())?;

        let mut code: Option<ID3DBlob> = None;
        let mut errors: Option<ID3DBlob> = None;

        let hr = D3DCompile(
            source.as_ptr() as *const std::ffi::c_void,
            source.len(),
            PCSTR::null(),
            None,
            None,
            PCSTR(entry.as_ptr() as *const u8),
            PCSTR(target.as_ptr() as *const u8),
            D3DCOMPILE_ENABLE_STRICTNESS,
            0,
            &mut code,
            Some(&mut errors),
        );

        if hr.is_err() {
            if let Some(err) = errors {
                let msg = std::slice::from_raw_parts(err.GetBufferPointer() as *const u8, err.GetBufferSize())
                    .iter()
                    .copied()
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>();
                let msg = String::from_utf8_lossy(&msg).to_string();
                return Err(format!("HLSL compile failed: {msg}"));
            }
            return Err(format!("D3DCompile failed: {hr:?}"));
        }

        code.ok_or_else(|| "D3DCompile succeeded but returned no blob".to_string())
    }
}

unsafe fn create_rtv(device: &ID3D11Device, swapchain: &IDXGISwapChain) -> Result<ID3D11RenderTargetView, String> {
    let back_buffer: ID3D11Texture2D = unsafe {
        swapchain
            .GetBuffer(0)
            .map_err(|e| format!("IDXGISwapChain::GetBuffer failed: {e:?}"))?
    };

    let mut rtv: Option<ID3D11RenderTargetView> = None;
    unsafe {
        device
            .CreateRenderTargetView(&back_buffer, None, Some(&mut rtv as *mut _))
            .map_err(|e| format!("CreateRenderTargetView failed: {e:?}"))?;
    }
    rtv.ok_or_else(|| "CreateRenderTargetView returned null RTV".to_string())
}
