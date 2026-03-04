use windows::Win32::Graphics::Direct3D11::*;

pub(crate) struct DirectX11Buffer {
    pub(super) buffer: Option<ID3D11Buffer>,
    pub(super) stride_bytes: u32,
}

impl DirectX11Buffer {
    pub(crate) fn destroy(&mut self) {
        self.buffer = None;
        self.stride_bytes = 0;
    }
}
