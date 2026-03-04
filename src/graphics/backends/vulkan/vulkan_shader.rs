use std::ffi::CString;

use ash::vk;

use shaderc::TargetEnv;

use crate::graphics::device::ShaderSource;
use crate::graphics::shader_compiler::{compile_to_spirv, ShaderStage};

pub(crate) struct VulkanShader {
    device: ash::Device,
    vert: vk::ShaderModule,
    frag: vk::ShaderModule,
}

impl VulkanShader {
    pub(super) fn from_sources(
        device: &ash::Device,
        vertex: ShaderSource<'_>,
        fragment: ShaderSource<'_>,
    ) -> Result<Self, String> {
        let vert_spv = compile_to_spirv(vertex, ShaderStage::Vertex, "shader.vert", TargetEnv::Vulkan)?;
        let frag_spv = compile_to_spirv(fragment, ShaderStage::Fragment, "shader.frag", TargetEnv::Vulkan)?;

        let vert = create_shader_module(device, &vert_spv)?;
        let frag = match create_shader_module(device, &frag_spv) {
            Ok(m) => m,
            Err(e) => {
                unsafe {
                    device.destroy_shader_module(vert, None);
                }
                return Err(e);
            }
        };

        Ok(Self {
            device: device.clone(),
            vert,
            frag,
        })
    }

    pub(super) fn stages<'a>(
        &'a self,
        entry_point: &'a CString,
    ) -> [vk::PipelineShaderStageCreateInfo<'a>; 2] {
        [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(self.vert)
                .name(entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(self.frag)
                .name(entry_point),
        ]
    }
}

impl Drop for VulkanShader {
    fn drop(&mut self) {
        unsafe {
            if self.vert != vk::ShaderModule::null() {
                self.device.destroy_shader_module(self.vert, None);
                self.vert = vk::ShaderModule::null();
            }
            if self.frag != vk::ShaderModule::null() {
                self.device.destroy_shader_module(self.frag, None);
                self.frag = vk::ShaderModule::null();
            }
        }
    }
}

pub(super) fn create_shader_module(device: &ash::Device, spv: &[u32]) -> Result<vk::ShaderModule, String> {
    let info = vk::ShaderModuleCreateInfo::default().code(spv);
    unsafe { device.create_shader_module(&info, None) }.map_err(|e| format!("vkCreateShaderModule failed: {e:?}"))
}
// use std::sync::{Arc, Mutex};

// use ash::vk;
// use shaderc::{Compiler, ShaderKind};

// use crate::graphics::shader::ShaderBackend;

// use super::vulkan_graphics::VulkanContext;

// /// Vulkan shader der ejer to VkShaderModule (vertex+fragment).
// /// bind()/unbind() er i første omgang en "state set" (til pipeline-creation senere).
// pub struct VulkanShader {
//     ctx: Arc<Mutex<VulkanContext>>,
//     pub(crate) vert: vk::ShaderModule,
//     pub(crate) frag: vk::ShaderModule,
// }

// impl VulkanShader {
//     pub fn from_source(
//         ctx: Arc<Mutex<VulkanContext>>,
//         vertex_source: &str,
//         fragment_source: &str,
//     ) -> Result<Self, String> {
//         let compiler = Compiler::new().ok_or("shaderc: failed to create compiler")?;

//         let vert_spv = compiler
//             .compile_into_spirv(vertex_source, ShaderKind::Vertex, "shader.vert", "main", None)
//             .map_err(|e| format!("shaderc vertex compile failed: {e}"))?;

//         let frag_spv = compiler
//             .compile_into_spirv(fragment_source, ShaderKind::Fragment, "shader.frag", "main", None)
//             .map_err(|e| format!("shaderc fragment compile failed: {e}"))?;

//         let device = {
//             let guard = ctx
//                 .lock()
//                 .map_err(|_| "VulkanContext mutex poisoned".to_string())?;
//             guard.device.clone()
//         };

//         let vert_info = vk::ShaderModuleCreateInfo::default().code(vert_spv.as_binary());
//         let frag_info = vk::ShaderModuleCreateInfo::default().code(frag_spv.as_binary());

//         let vert = unsafe { device.create_shader_module(&vert_info, None) }
//             .map_err(|e| format!("vkCreateShaderModule (vert) failed: {e:?}"))?;
//         let frag = unsafe { device.create_shader_module(&frag_info, None) }
//             .map_err(|e| format!("vkCreateShaderModule (frag) failed: {e:?}"))?;

//         Ok(Self { ctx, vert, frag })
//     }
// }

// impl ShaderBackend for VulkanShader {
//     fn bind(&self) {
//         // TODO: Gem "current shader" i VulkanContext til pipeline-creation (hvis du vil beholde API).
//         let _ = &self;
//     }

//     fn unbind(&self) {
//         // TODO: clear current shader i context
//         let _ = &self;
//     }
// }

// impl Drop for VulkanShader {
//     fn drop(&mut self) {
//         // Best-effort: ingen panics i Drop.
//         let Ok(guard) = self.ctx.lock() else { return; };
//         let device = &guard.device;

//         unsafe {
//             device.destroy_shader_module(self.vert, None);
//             device.destroy_shader_module(self.frag, None);
//         }
//     }
// }