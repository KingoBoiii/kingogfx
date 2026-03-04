use windows::Win32::Graphics::Direct3D12::*;

pub(crate) struct DirectX12Buffer {
    pub(super) resource: Option<ID3D12Resource>,
    pub(super) size_bytes: u32,
    pub(super) stride_bytes: u32,
}

impl DirectX12Buffer {
    pub(crate) fn destroy(&mut self) {
        self.resource = None;
        self.size_bytes = 0;
        self.stride_bytes = 0;
    }
}
