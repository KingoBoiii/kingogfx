use shaderc::{CompileOptions, Compiler, ShaderKind, SourceLanguage, TargetEnv};

use crate::graphics::device::{ShaderLanguage, ShaderSource};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    fn to_shaderc_kind(self) -> ShaderKind {
        match self {
            ShaderStage::Vertex => ShaderKind::Vertex,
            ShaderStage::Fragment => ShaderKind::Fragment,
        }
    }
}

pub(crate) fn compile_to_spirv(
    src: ShaderSource<'_>,
    stage: ShaderStage,
    file_name: &str,
    target_env: TargetEnv,
) -> Result<Vec<u32>, String> {
    let compiler = Compiler::new().map_err(|e| format!("shaderc: failed to create compiler: {e:?}"))?;

    let mut options =
        CompileOptions::new().map_err(|e| format!("shaderc: failed to create compile options: {e:?}"))?;

    options.set_auto_map_locations(true);
    options.set_target_env(target_env, 0);

    match src.language {
        ShaderLanguage::Glsl => {}
        ShaderLanguage::Hlsl => {
            options.set_source_language(SourceLanguage::HLSL);
        }
    }

    let artifact = compiler
        .compile_into_spirv(src.source, stage.to_shaderc_kind(), file_name, "main", Some(&options))
        .map_err(|e| format!("shaderc compile failed: {e}"))?;

    Ok(artifact.as_binary().to_vec())
}

pub(crate) fn spirv_to_hlsl(spv: &[u32]) -> Result<String, String> {
    let spv_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(spv.as_ptr() as *const u8, spv.len() * std::mem::size_of::<u32>())
    };

    let module = naga::front::spv::parse_u8_slice(
        spv_bytes,
        &naga::front::spv::Options {
            adjust_coordinate_space: false,
            strict_capabilities: false,
            block_ctx_dump_prefix: None,
        },
    )
    .map_err(|e| format!("naga spv parse failed: {e:?}"))?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let info = validator
        .validate(&module)
        .map_err(|e| format!("naga validation failed: {e:?}"))?;

    let options = naga::back::hlsl::Options {
        shader_model: naga::back::hlsl::ShaderModel::V5_0,
        ..Default::default()
    };

    let mut output = String::new();
    let mut writer = naga::back::hlsl::Writer::new(&mut output, &options);
    writer
        .write(&module, &info)
        .map_err(|e| format!("naga hlsl write failed: {e:?}"))?;

    Ok(output)
}
