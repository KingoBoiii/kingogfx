use std::ffi::CString;
use std::sync::Arc;

use ash::vk;

use super::VulkanShader;

pub(crate) struct VulkanPipeline {
    pub(super) device: ash::Device,
    pub(super) pipeline: vk::Pipeline,
    pub(super) layout: vk::PipelineLayout,
    pub(super) _shader: Arc<VulkanShader>,
}

impl VulkanPipeline {
    pub(crate) fn destroy(&mut self) {
        if self.pipeline == vk::Pipeline::null() {
            return;
        }
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
        self.pipeline = vk::Pipeline::null();
        self.layout = vk::PipelineLayout::null();
    }
}

pub(super) fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    shader: &Arc<VulkanShader>,
) -> Result<VulkanPipeline, String> {
    let main = CString::new("main").unwrap();
    let stages = shader.stages(&main);

    let binding_desc = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride((2 * std::mem::size_of::<f32>()) as u32)
        .input_rate(vk::VertexInputRate::VERTEX);

    let attr_desc = vk::VertexInputAttributeDescription::default()
        .location(0)
        .binding(0)
        .format(vk::Format::R32G32_SFLOAT)
        .offset(0);

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding_desc))
        .vertex_attribute_descriptions(std::slice::from_ref(&attr_desc));

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let raster = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        // Vulkan NDC uses inverted Y compared to OpenGL, which effectively flips winding
        // for simple clip-space triangles like the example.
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1.0);

    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(false)
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        );

    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .attachments(std::slice::from_ref(&color_blend_attachment));

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let layout_info = vk::PipelineLayoutCreateInfo::default();
    let layout = unsafe {
        device
            .create_pipeline_layout(&layout_info, None)
            .map_err(|e| format!("vkCreatePipelineLayout failed: {e:?}"))?
    };

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&raster)
        .multisample_state(&multisample)
        .color_blend_state(&color_blend)
        .dynamic_state(&dynamic)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|(_, e)| format!("vkCreateGraphicsPipelines failed: {e:?}"))?
    }[0];

    Ok(VulkanPipeline {
        device: device.clone(),
        pipeline,
        layout,
        _shader: Arc::clone(shader),
    })
}
