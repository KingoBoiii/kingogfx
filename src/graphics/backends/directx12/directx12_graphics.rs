use std::ffi::CString;
use std::sync::Arc;

use glfw_sys::GLFWwindow;

use windows::Win32::Foundation::*;
use windows::core::{Interface, PCSTR, PCWSTR};
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Direct3D::Fxc::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::System::Threading::CreateEventW;

use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
use crate::window::Window;

use super::directx12_swapchain::{DirectX12Swapchain, FRAME_COUNT};
use super::{DirectX12Buffer, DirectX12Pipeline, DirectX12Shader};

pub(crate) struct DirectX12Graphics {
    device: ID3D12Device,
    queue: ID3D12CommandQueue,

    swapchain: DirectX12Swapchain,

    allocators: [ID3D12CommandAllocator; FRAME_COUNT],
    cmd_list: ID3D12GraphicsCommandList,

    viewport: Option<D3D12_VIEWPORT>,
    scissor: Option<RECT>,

    in_frame: bool,
}

impl DirectX12Graphics {
    pub(crate) fn create(window: &mut Window) -> Result<Self, String> {
        let hwnd = hwnd_from_window(window)?;

        unsafe {
            let factory: IDXGIFactory4 = CreateDXGIFactory1()
                .map_err(|e| format!("CreateDXGIFactory1 failed: {e:?}"))?;

            let adapter = pick_hardware_adapter(&factory)?;

            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)
                .map_err(|e| format!("D3D12CreateDevice failed: {e:?}"))?;
            let device = device.ok_or_else(|| "D3D12CreateDevice returned null device".to_string())?;

            let queue_desc = D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0 as i32,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                NodeMask: 0,
            };
            let queue = device
                .CreateCommandQueue(&queue_desc)
                .map_err(|e| format!("CreateCommandQueue failed: {e:?}"))?;

            let (width, height) = framebuffer_size_u32(window);

            let swapchain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                Stereo: BOOL(0),
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: FRAME_COUNT as u32,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                Flags: 0,
            };

            let swapchain1 = factory
                .CreateSwapChainForHwnd(&queue, hwnd, &swapchain_desc, None, None)
                .map_err(|e| format!("CreateSwapChainForHwnd failed: {e:?}"))?;

            // Disable Alt+Enter fullscreen.
            let _ = factory.MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER);

            let swapchain: IDXGISwapChain3 = swapchain1
                .cast()
                .map_err(|e| format!("Swapchain cast to IDXGISwapChain3 failed: {e:?}"))?;

            let rtv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: FRAME_COUNT as u32,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };
            let rtv_heap = device
                .CreateDescriptorHeap(&rtv_heap_desc)
                .map_err(|e| format!("CreateDescriptorHeap(RTV) failed: {e:?}"))?;

            let rtv_increment = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

            let mut render_targets: [Option<ID3D12Resource>; FRAME_COUNT] = [None, None];
            for i in 0..FRAME_COUNT {
                let buffer: ID3D12Resource = swapchain
                    .GetBuffer(i as u32)
                    .map_err(|e| format!("IDXGISwapChain3::GetBuffer({i}) failed: {e:?}"))?;
                let handle = cpu_handle_at(&rtv_heap, rtv_increment, i);
                device.CreateRenderTargetView(&buffer, None, handle);
                render_targets[i] = Some(buffer);
            }

            let fence = device
                .CreateFence(0, D3D12_FENCE_FLAG_NONE)
                .map_err(|e| format!("CreateFence failed: {e:?}"))?;

            let fence_event = CreateEventW(None, false, false, PCWSTR::null())
                .map_err(|e| format!("CreateEventW failed: {e:?}"))?;

            let frame_index = swapchain.GetCurrentBackBufferIndex() as usize;

            let allocators: [ID3D12CommandAllocator; FRAME_COUNT] = [
                device
                    .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
                    .map_err(|e| format!("CreateCommandAllocator[0] failed: {e:?}"))?,
                device
                    .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
                    .map_err(|e| format!("CreateCommandAllocator[1] failed: {e:?}"))?,
            ];

            let cmd_list: ID3D12GraphicsCommandList = device
                .CreateCommandList(
                    0,
                    D3D12_COMMAND_LIST_TYPE_DIRECT,
                    &allocators[frame_index],
                    None,
                )
                .map_err(|e| format!("CreateCommandList failed: {e:?}"))?;
            // Close it; it will be reset per-frame.
            cmd_list.Close().ok();

            let swapchain = DirectX12Swapchain {
                swapchain,
                rtv_heap,
                rtv_increment,
                render_targets,
                frame_index,
                fence,
                fence_values: [0, 0],
                fence_event,
                width,
                height,
            };

            Ok(Self {
                device,
                queue,
                swapchain,
                allocators,
                cmd_list,
                viewport: None,
                scissor: None,
                in_frame: false,
            })
        }
    }

    pub(crate) fn shutdown(&mut self, _window: &mut Window) -> Result<(), String> {
        // Ensure GPU is idle before dropping.
        self.wait_gpu_idle()?;
        Ok(())
    }

    pub(crate) fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let w = (width.max(0)) as f32;
        let h = (height.max(0)) as f32;
        self.viewport = Some(D3D12_VIEWPORT {
            TopLeftX: x as f32,
            TopLeftY: y as f32,
            Width: w,
            Height: h,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        });
        self.scissor = Some(RECT {
            left: x,
            top: y,
            right: x + width.max(0),
            bottom: y + height.max(0),
        });
    }

    pub(crate) fn create_buffer_init(&mut self, data: &[f32], usage: BufferUsage) -> Result<DirectX12Buffer, String> {
        if data.is_empty() {
            return Err("buffer data is empty".to_string());
        }
        match usage {
            BufferUsage::Vertex => {}
        }

        let size_bytes = (data.len() * std::mem::size_of::<f32>()) as u32;
        let stride_bytes = (2 * std::mem::size_of::<f32>()) as u32;

        unsafe {
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_UPLOAD,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: 0,
                Width: size_bytes as u64,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_UNKNOWN,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                Flags: D3D12_RESOURCE_FLAG_NONE,
            };

            let mut resource: Option<ID3D12Resource> = None;
            self.device
                .CreateCommittedResource(
                    &heap_props,
                    D3D12_HEAP_FLAG_NONE,
                    &desc,
                    D3D12_RESOURCE_STATE_GENERIC_READ,
                    None,
                    &mut resource,
                )
                .map_err(|e| format!("CreateCommittedResource (upload buffer) failed: {e:?}"))?;
            let resource = resource.ok_or_else(|| "CreateCommittedResource returned null resource".to_string())?;

            let mut mapped: *mut std::ffi::c_void = std::ptr::null_mut();
            resource
                .Map(0, None, Some(&mut mapped))
                .map_err(|e| format!("ID3D12Resource::Map failed: {e:?}"))?;
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                mapped as *mut u8,
                size_bytes as usize,
            );
            resource.Unmap(0, None);

            Ok(DirectX12Buffer {
                resource: Some(resource),
                size_bytes,
                stride_bytes,
            })
        }
    }

    pub(crate) fn create_shader(&mut self, desc: ShaderDescriptor<'_>) -> Result<Arc<DirectX12Shader>, String> {
        let vs_src = desc
            .vertex_source_hlsl
            .ok_or_else(|| "DirectX12 shader creation requires vertex_source_hlsl".to_string())?;
        let ps_src = desc
            .fragment_source_hlsl
            .ok_or_else(|| "DirectX12 shader creation requires fragment_source_hlsl".to_string())?;

        let vs = compile_hlsl(vs_src, "main", "vs_5_0")?;
        let ps = compile_hlsl(ps_src, "main", "ps_5_0")?;

        Ok(Arc::new(DirectX12Shader { vs, ps }))
    }

    pub(crate) fn create_pipeline(&mut self, shader: &Arc<DirectX12Shader>) -> Result<DirectX12Pipeline, String> {
        unsafe {
            let root_signature = create_root_signature(&self.device)?;

            static SEMANTIC_POSITION: &[u8] = b"POSITION\0";
            let input_element = D3D12_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(SEMANTIC_POSITION.as_ptr()),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            };
            let input_layout = D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: &input_element,
                NumElements: 1,
            };

            let vs = D3D12_SHADER_BYTECODE {
                pShaderBytecode: shader.vs.GetBufferPointer(),
                BytecodeLength: shader.vs.GetBufferSize(),
            };
            let ps = D3D12_SHADER_BYTECODE {
                pShaderBytecode: shader.ps.GetBufferPointer(),
                BytecodeLength: shader.ps.GetBufferSize(),
            };

            let rasterizer = D3D12_RASTERIZER_DESC {
                FillMode: D3D12_FILL_MODE_SOLID,
                CullMode: D3D12_CULL_MODE_NONE,
                FrontCounterClockwise: BOOL(0),
                DepthBias: D3D12_DEFAULT_DEPTH_BIAS as i32,
                DepthBiasClamp: D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
                SlopeScaledDepthBias: D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
                DepthClipEnable: BOOL(1),
                MultisampleEnable: BOOL(0),
                AntialiasedLineEnable: BOOL(0),
                ForcedSampleCount: 0,
                ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
            };

            let blend = D3D12_BLEND_DESC {
                AlphaToCoverageEnable: BOOL(0),
                IndependentBlendEnable: BOOL(0),
                RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: BOOL(0),
                    LogicOpEnable: BOOL(0),
                    SrcBlend: D3D12_BLEND_ONE,
                    DestBlend: D3D12_BLEND_ZERO,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_ONE,
                    DestBlendAlpha: D3D12_BLEND_ZERO,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_NOOP,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                }; 8],
            };

            let pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::ManuallyDrop::new(Some(root_signature.clone())),
                VS: vs,
                PS: ps,
                DS: D3D12_SHADER_BYTECODE::default(),
                HS: D3D12_SHADER_BYTECODE::default(),
                GS: D3D12_SHADER_BYTECODE::default(),
                StreamOutput: D3D12_STREAM_OUTPUT_DESC::default(),
                BlendState: blend,
                SampleMask: u32::MAX,
                RasterizerState: rasterizer,
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: BOOL(0),
                    StencilEnable: BOOL(0),
                    ..Default::default()
                },
                InputLayout: input_layout,
                IBStripCutValue: D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED,
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                RTVFormats: {
                    let mut fmts = [DXGI_FORMAT_UNKNOWN; 8];
                    fmts[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
                    fmts
                },
                DSVFormat: DXGI_FORMAT_UNKNOWN,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                NodeMask: 0,
                CachedPSO: D3D12_CACHED_PIPELINE_STATE::default(),
                Flags: D3D12_PIPELINE_STATE_FLAG_NONE,
            };

            let pso = self
                .device
                .CreateGraphicsPipelineState(&pso_desc)
                .map_err(|e| format!("CreateGraphicsPipelineState failed: {e:?}"))?;

            Ok(DirectX12Pipeline {
                root_signature,
                pso,
            })
        }
    }

    pub(crate) fn begin_frame(&mut self, window: &mut Window, clear: ClearColor) -> Result<(), String> {
        if self.in_frame {
            return Err("begin_frame called while already in a frame".to_string());
        }

        self.resize_if_needed(window)?;

        self.swapchain.wait_for_frame()?;

        unsafe {
            let allocator = &self.allocators[self.swapchain.frame_index];
            allocator
                .Reset()
                .map_err(|e| format!("ID3D12CommandAllocator::Reset failed: {e:?}"))?;
            self.cmd_list
                .Reset(allocator, None)
                .map_err(|e| format!("ID3D12GraphicsCommandList::Reset failed: {e:?}"))?;

            let back_buffer = self.swapchain.render_targets[self.swapchain.frame_index]
                .as_ref()
                .ok_or_else(|| "Back buffer is null".to_string())?;

            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::ManuallyDrop::new(Some(back_buffer.clone())),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                        StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                    }),
                },
            };
            self.cmd_list.ResourceBarrier(&[barrier]);

            let rtv = self.swapchain.rtv_handle_for_index(self.swapchain.frame_index);
            self.cmd_list.OMSetRenderTargets(1, Some(&rtv), false, None);

            let color = [clear.r, clear.g, clear.b, clear.a];
            self.cmd_list.ClearRenderTargetView(rtv, &color, None);

            let vp = self.viewport.unwrap_or(D3D12_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: self.swapchain.width as f32,
                Height: self.swapchain.height as f32,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            });
            let sc = self.scissor.unwrap_or(RECT {
                left: 0,
                top: 0,
                right: self.swapchain.width as i32,
                bottom: self.swapchain.height as i32,
            });
            self.cmd_list.RSSetViewports(&[vp]);
            self.cmd_list.RSSetScissorRects(&[sc]);
        }

        self.in_frame = true;
        Ok(())
    }

    pub(crate) fn set_pipeline(&mut self, pipeline: &DirectX12Pipeline) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_pipeline must be called between begin_frame/end_frame".to_string());
        }

        unsafe {
            self.cmd_list.SetGraphicsRootSignature(&pipeline.root_signature);
            self.cmd_list.SetPipelineState(&pipeline.pso);
            self.cmd_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        }
        Ok(())
    }

    pub(crate) fn set_vertex_buffer(&mut self, slot: u32, buffer: &DirectX12Buffer) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_vertex_buffer must be called between begin_frame/end_frame".to_string());
        }
        if slot != 0 {
            return Err("DirectX12 backend currently supports only slot 0".to_string());
        }

        let resource = buffer
            .resource
            .as_ref()
            .ok_or_else(|| "DirectX12 buffer resource is null".to_string())?;

        unsafe {
            let vbv = D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: resource.GetGPUVirtualAddress(),
                SizeInBytes: buffer.size_bytes,
                StrideInBytes: buffer.stride_bytes,
            };
            self.cmd_list.IASetVertexBuffers(0, Some(&[vbv]));
        }

        Ok(())
    }

    pub(crate) fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), String> {
        if !self.in_frame {
            return Err("draw must be called between begin_frame/end_frame".to_string());
        }

        unsafe {
            self.cmd_list.DrawInstanced(vertex_count, 1, first_vertex, 0);
        }
        Ok(())
    }

    pub(crate) fn end_frame(&mut self, _window: &mut Window) -> Result<(), String> {
        if !self.in_frame {
            return Err("end_frame called without begin_frame".to_string());
        }

        unsafe {
            let back_buffer = self.swapchain.render_targets[self.swapchain.frame_index]
                .as_ref()
                .ok_or_else(|| "Back buffer is null".to_string())?;

            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::ManuallyDrop::new(Some(back_buffer.clone())),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: D3D12_RESOURCE_STATE_PRESENT,
                    }),
                },
            };
            self.cmd_list.ResourceBarrier(&[barrier]);

            self.cmd_list
                .Close()
                .map_err(|e| format!("ID3D12GraphicsCommandList::Close failed: {e:?}"))?;

            let list: ID3D12CommandList = self.cmd_list.cast().unwrap();
            self.queue.ExecuteCommandLists(&[Some(list)]);

            self.swapchain
                .swapchain
                .Present(1, DXGI_PRESENT(0))
                .ok()
                .map_err(|e| format!("IDXGISwapChain::Present failed: {e:?}"))?;

            self.swapchain.signal(&self.queue)?;
            self.swapchain.advance_frame();
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

        self.wait_gpu_idle()?;

        unsafe {
            for rt in &mut self.swapchain.render_targets {
                *rt = None;
            }

            self.swapchain
                .swapchain
                .ResizeBuffers(
                    FRAME_COUNT as u32,
                    w,
                    h,
                    DXGI_FORMAT_R8G8B8A8_UNORM,
                    DXGI_SWAP_CHAIN_FLAG(0),
                )
                .map_err(|e| format!("IDXGISwapChain3::ResizeBuffers failed: {e:?}"))?;

            self.swapchain.width = w;
            self.swapchain.height = h;
            self.swapchain.frame_index = self.swapchain.swapchain.GetCurrentBackBufferIndex() as usize;

            for i in 0..FRAME_COUNT {
                let buffer: ID3D12Resource = self.swapchain
                    .swapchain
                    .GetBuffer(i as u32)
                    .map_err(|e| format!("IDXGISwapChain3::GetBuffer({i}) failed: {e:?}"))?;
                let handle = self.swapchain.rtv_handle_for_index(i);
                self.device.CreateRenderTargetView(&buffer, None, handle);
                self.swapchain.render_targets[i] = Some(buffer);
            }
        }

        Ok(())
    }

    fn wait_gpu_idle(&mut self) -> Result<(), String> {
        // Signal and wait for current frame fence.
        self.swapchain.signal(&self.queue)?;
        self.swapchain.wait_for_frame()?;
        Ok(())
    }
}

fn cpu_handle_at(heap: &ID3D12DescriptorHeap, increment: u32, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
    unsafe {
        let base = heap.GetCPUDescriptorHandleForHeapStart();
        D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: base.ptr + (index as usize * increment as usize) as usize,
        }
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

unsafe fn pick_hardware_adapter(factory: &IDXGIFactory4) -> Result<IDXGIAdapter1, String> {
    let mut i = 0;
    loop {
        let adapter = match unsafe { factory.EnumAdapters1(i) } {
            Ok(a) => a,
            Err(_) => break,
        };

        let desc = unsafe {
            adapter
                .GetDesc1()
                .map_err(|e| format!("IDXGIAdapter1::GetDesc1 failed: {e:?}"))?
        };

        // Skip software adapters.
        if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 {
            i += 1;
            continue;
        }

        // Probe if D3D12 device can be created.
        let mut tmp: Option<ID3D12Device> = None;
        if unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut tmp) }.is_ok() {
            return Ok(adapter);
        }

        i += 1;
    }

    // Fallback to WARP.
    unsafe {
        factory
            .EnumWarpAdapter::<IDXGIAdapter1>()
            .map_err(|e| format!("EnumWarpAdapter failed: {e:?}"))
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

fn create_root_signature(device: &ID3D12Device) -> Result<ID3D12RootSignature, String> {
    unsafe {
        let desc = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 0,
            pParameters: std::ptr::null(),
            NumStaticSamplers: 0,
            pStaticSamplers: std::ptr::null(),
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };

        let mut blob: Option<ID3DBlob> = None;
        let mut error: Option<ID3DBlob> = None;

        D3D12SerializeRootSignature(&desc, D3D_ROOT_SIGNATURE_VERSION_1, &mut blob, Some(&mut error))
            .map_err(|e| {
                if let Some(err) = error {
                    let msg = std::slice::from_raw_parts(err.GetBufferPointer() as *const u8, err.GetBufferSize());
                    let msg = String::from_utf8_lossy(msg).trim_end_matches('\0').to_string();
                    return format!("D3D12SerializeRootSignature failed: {msg}");
                }
                format!("D3D12SerializeRootSignature failed: {e:?}")
            })?;

        let blob = blob.ok_or_else(|| "D3D12SerializeRootSignature returned null blob".to_string())?;

        device
            .CreateRootSignature(0, std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize()))
            .map_err(|e| format!("CreateRootSignature failed: {e:?}"))
    }
}
