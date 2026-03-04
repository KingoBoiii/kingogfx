use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

pub(super) struct DirectX11Swapchain {
    pub(super) swapchain: IDXGISwapChain,
    pub(super) rtv: ID3D11RenderTargetView,
    pub(super) width: u32,
    pub(super) height: u32,
}
