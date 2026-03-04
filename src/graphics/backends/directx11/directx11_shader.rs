use windows::Win32::Graphics::Direct3D::*;

pub(crate) struct DirectX11Shader {
    pub(super) vs: ID3DBlob,
    pub(super) ps: ID3DBlob,
}
