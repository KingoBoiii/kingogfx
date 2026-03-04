use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Threading::*;

pub(super) const FRAME_COUNT: usize = 2;

pub(super) struct DirectX12Swapchain {
    pub(super) swapchain: IDXGISwapChain3,
    pub(super) rtv_heap: ID3D12DescriptorHeap,
    pub(super) rtv_increment: u32,
    pub(super) render_targets: [Option<ID3D12Resource>; FRAME_COUNT],
    pub(super) frame_index: usize,

    pub(super) fence: ID3D12Fence,
    pub(super) fence_values: [u64; FRAME_COUNT],
    pub(super) fence_event: HANDLE,

    pub(super) width: u32,
    pub(super) height: u32,
}

impl DirectX12Swapchain {
    pub(super) fn wait_for_frame(&mut self) -> Result<(), String> {
        let value = self.fence_values[self.frame_index];
        unsafe {
            if self.fence.GetCompletedValue() < value {
                self.fence
                    .SetEventOnCompletion(value, self.fence_event)
                    .map_err(|e| format!("ID3D12Fence::SetEventOnCompletion failed: {e:?}"))?;
                WaitForSingleObject(self.fence_event, u32::MAX);
            }
        }
        Ok(())
    }

    pub(super) fn signal(&mut self, queue: &ID3D12CommandQueue) -> Result<(), String> {
        self.fence_values[self.frame_index] += 1;
        let value = self.fence_values[self.frame_index];
        unsafe {
            queue
                .Signal(&self.fence, value)
                .map_err(|e| format!("ID3D12CommandQueue::Signal failed: {e:?}"))?;
        }
        Ok(())
    }

    pub(super) fn advance_frame(&mut self) {
        unsafe {
            self.frame_index = self.swapchain.GetCurrentBackBufferIndex() as usize;
        }
    }

    pub(super) fn rtv_handle_for_index(&self, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe {
            let base = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: base.ptr + (index as usize * self.rtv_increment as usize) as usize,
            }
        }
    }

    pub(super) fn destroy(&mut self) {
        for rt in &mut self.render_targets {
            *rt = None;
        }
        unsafe {
            if !self.fence_event.is_invalid() {
                let _ = CloseHandle(self.fence_event);
                self.fence_event = HANDLE::default();
            }
        }
    }
}

impl Drop for DirectX12Swapchain {
    fn drop(&mut self) {
        self.destroy();
    }
}
