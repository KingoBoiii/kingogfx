use windows::Win32::Graphics::Direct3D12::*;

pub(crate) struct DirectX12Pipeline {
    pub(super) root_signature: ID3D12RootSignature,
    pub(super) pso: ID3D12PipelineState,
}

impl DirectX12Pipeline {
    pub(crate) fn destroy(&mut self) {
        // COM objects are released automatically.
    }
}
