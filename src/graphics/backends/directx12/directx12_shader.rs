use windows::Win32::Graphics::Direct3D::*;

pub(crate) struct DirectX12Shader {
    pub(super) vs: ID3DBlob,
    pub(super) ps: ID3DBlob,
}
