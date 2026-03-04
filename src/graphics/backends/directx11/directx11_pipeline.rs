use windows::Win32::Graphics::Direct3D11::*;

pub(crate) struct DirectX11Pipeline {
    pub(super) vs: ID3D11VertexShader,
    pub(super) ps: ID3D11PixelShader,
    pub(super) input_layout: ID3D11InputLayout,
    pub(super) raster_state: ID3D11RasterizerState,
}

impl DirectX11Pipeline {
    pub(crate) fn destroy(&mut self) {
        // COM objects are released automatically.
    }
}
