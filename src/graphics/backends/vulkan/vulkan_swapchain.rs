use ash::{vk, Entry};
use ash::vk::Handle;

use crate::window::Window;

pub(super) struct VulkanSurface {
    surface_loader: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
}

impl VulkanSurface {
    pub(super) fn create(entry: &Entry, instance: &ash::Instance, window: &mut Window) -> Result<Self, String> {
        let surface_loader = ash::khr::surface::Instance::new(entry, instance);
        let surface = create_surface(instance, window)?;
        Ok(Self {
            surface_loader,
            surface,
        })
    }

    pub(super) fn surface_loader(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub(super) fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub(super) fn recreate(&mut self, instance: &ash::Instance, window: &mut Window) -> Result<(), String> {
        unsafe {
            if self.surface != vk::SurfaceKHR::null() {
                self.surface_loader.destroy_surface(self.surface, None);
                self.surface = vk::SurfaceKHR::null();
            }
        }
        self.surface = create_surface(instance, window)?;
        Ok(())
    }

    pub(super) fn destroy(&mut self) {
        unsafe {
            if self.surface != vk::SurfaceKHR::null() {
                self.surface_loader.destroy_surface(self.surface, None);
                self.surface = vk::SurfaceKHR::null();
            }
        }
    }
}

pub(super) struct VulkanSwapchain {
    surface: VulkanSurface,

    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,

    swapchain_dirty: bool,
    surface_lost: bool,
}

impl VulkanSwapchain {
    pub(super) fn create(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        surface: VulkanSurface,
        window: &mut Window,
        queue_family_index: u32,
    ) -> Result<Self, String> {
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);
        let (swapchain, swapchain_images, swapchain_format, swapchain_extent) = create_swapchain(
            physical_device,
            device,
            surface.surface_loader(),
            surface.surface(),
            &swapchain_loader,
            window,
            queue_family_index,
        )?;

        let swapchain_image_views = create_image_views(device, &swapchain_images, swapchain_format)?;
        let render_pass = create_render_pass(device, swapchain_format)?;
        let framebuffers = create_framebuffers(device, render_pass, &swapchain_image_views, swapchain_extent)?;

        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .map_err(|e| format!("vkCreateCommandPool failed: {e:?}"))?
        };

        let command_buffers = allocate_command_buffers(device, command_pool, swapchain_images.len() as u32)?;

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available = unsafe {
            device
                .create_semaphore(&sem_info, None)
                .map_err(|e| format!("vkCreateSemaphore failed: {e:?}"))?
        };
        let render_finished = unsafe {
            device
                .create_semaphore(&sem_info, None)
                .map_err(|e| format!("vkCreateSemaphore failed: {e:?}"))?
        };
        let in_flight = unsafe {
            device
                .create_fence(&fence_info, None)
                .map_err(|e| format!("vkCreateFence failed: {e:?}"))?
        };

        Ok(Self {
            surface,
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            swapchain_format,
            swapchain_extent,
            render_pass,
            framebuffers,
            command_pool,
            command_buffers,

            image_available,
            render_finished,
            in_flight,

            swapchain_dirty: false,
            surface_lost: false,
        })
    }

    pub(super) fn image_available(&self) -> vk::Semaphore {
        self.image_available
    }

    pub(super) fn render_finished(&self) -> vk::Semaphore {
        self.render_finished
    }

    pub(super) fn in_flight_fence(&self) -> vk::Fence {
        self.in_flight
    }

    pub(super) fn extent(&self) -> vk::Extent2D {
        self.swapchain_extent
    }

    pub(super) fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.swapchain_dirty
    }

    pub(super) fn is_surface_lost(&self) -> bool {
        self.surface_lost
    }

    pub(super) fn mark_dirty(&mut self) {
        self.swapchain_dirty = true;
    }

    pub(super) fn mark_surface_lost(&mut self) {
        self.swapchain_dirty = true;
        self.surface_lost = true;
    }

    pub(super) fn command_buffer(&self, image_index: u32) -> vk::CommandBuffer {
        self.command_buffers[image_index as usize]
    }

    pub(super) fn framebuffer(&self, image_index: u32) -> vk::Framebuffer {
        self.framebuffers[image_index as usize]
    }

    pub(super) fn acquire_next_image(&self) -> Result<(u32, bool), vk::Result> {
        unsafe {
            self.swapchain_loader
                .acquire_next_image(self.swapchain, u64::MAX, self.image_available, vk::Fence::null())
        }
    }

    pub(super) fn present(
        &mut self,
        queue: vk::Queue,
        image_index: u32,
    ) -> Result<(), String> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let wait = [self.render_finished];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            match self.swapchain_loader.queue_present(queue, &present_info) {
                Ok(suboptimal) => {
                    if suboptimal {
                        self.swapchain_dirty = true;
                    }
                    Ok(())
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.swapchain_dirty = true;
                    Ok(())
                }
                Err(vk::Result::ERROR_SURFACE_LOST_KHR) => {
                    self.swapchain_dirty = true;
                    self.surface_lost = true;
                    Ok(())
                }
                Err(e) => Err(format!("vkQueuePresentKHR failed: {e:?}")),
            }
        }
    }

    pub(super) fn recreate(
        &mut self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        window: &mut Window,
        queue_family_index: u32,
        recreate_surface: bool,
    ) -> Result<(), String> {
        let old_format = self.swapchain_format;

        // If minimized, don't try to recreate. Let caller retry later.
        let (fb_w, fb_h) = window.framebuffer_size();
        if fb_w <= 0 || fb_h <= 0 {
            return Err("Framebuffer size is 0 (minimized)".to_string());
        }

        unsafe {
            for fb in &self.framebuffers {
                device.destroy_framebuffer(*fb, None);
            }
            self.framebuffers.clear();

            for view in &self.swapchain_image_views {
                device.destroy_image_view(*view, None);
            }
            self.swapchain_image_views.clear();

            if !self.command_buffers.is_empty() {
                device.free_command_buffers(self.command_pool, &self.command_buffers);
                self.command_buffers.clear();
            }

            if self.swapchain != vk::SwapchainKHR::null() {
                self.swapchain_loader.destroy_swapchain(self.swapchain, None);
                self.swapchain = vk::SwapchainKHR::null();
            }
        }

        if recreate_surface {
            self.surface.recreate(instance, window)?;
        }

        let (swapchain, images, format, extent) = create_swapchain(
            physical_device,
            device,
            self.surface.surface_loader(),
            self.surface.surface(),
            &self.swapchain_loader,
            window,
            queue_family_index,
        )?;

        self.swapchain = swapchain;
        self.swapchain_images = images;
        self.swapchain_format = format;
        self.swapchain_extent = extent;

        self.swapchain_image_views = create_image_views(device, &self.swapchain_images, self.swapchain_format)?;

        // Only recreate the render pass if the swapchain format changed.
        // Keeping the render pass stable allows pipelines created once at startup
        // to remain valid across resizes.
        if self.render_pass == vk::RenderPass::null() || self.swapchain_format != old_format {
            unsafe {
                if self.render_pass != vk::RenderPass::null() {
                    device.destroy_render_pass(self.render_pass, None);
                }
            }
            self.render_pass = create_render_pass(device, self.swapchain_format)?;
        }

        self.framebuffers = create_framebuffers(device, self.render_pass, &self.swapchain_image_views, self.swapchain_extent)?;
        self.command_buffers = allocate_command_buffers(device, self.command_pool, self.swapchain_images.len() as u32)?;

        self.swapchain_dirty = false;
        self.surface_lost = false;
        Ok(())
    }

    pub(super) fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            if self.in_flight != vk::Fence::null() {
                device.destroy_fence(self.in_flight, None);
                self.in_flight = vk::Fence::null();
            }
            if self.render_finished != vk::Semaphore::null() {
                device.destroy_semaphore(self.render_finished, None);
                self.render_finished = vk::Semaphore::null();
            }
            if self.image_available != vk::Semaphore::null() {
                device.destroy_semaphore(self.image_available, None);
                self.image_available = vk::Semaphore::null();
            }

            if self.command_pool != vk::CommandPool::null() {
                if !self.command_buffers.is_empty() {
                    device.free_command_buffers(self.command_pool, &self.command_buffers);
                    self.command_buffers.clear();
                }
                device.destroy_command_pool(self.command_pool, None);
                self.command_pool = vk::CommandPool::null();
            }

            for fb in &self.framebuffers {
                device.destroy_framebuffer(*fb, None);
            }
            self.framebuffers.clear();

            if self.render_pass != vk::RenderPass::null() {
                device.destroy_render_pass(self.render_pass, None);
                self.render_pass = vk::RenderPass::null();
            }

            for view in &self.swapchain_image_views {
                device.destroy_image_view(*view, None);
            }
            self.swapchain_image_views.clear();

            if self.swapchain != vk::SwapchainKHR::null() {
                self.swapchain_loader.destroy_swapchain(self.swapchain, None);
                self.swapchain = vk::SwapchainKHR::null();
            }
        }

        self.surface.destroy();
    }
}

fn create_surface(instance: &ash::Instance, window: &mut Window) -> Result<vk::SurfaceKHR, String> {
    let glfw_window = window.glfw_window_ptr();
    if glfw_window.is_null() {
        return Err("Window backend did not provide a valid GLFWwindow pointer".to_string());
    }

    let raw_instance: glfw_sys::VkInstance = instance.handle().as_raw() as usize as glfw_sys::VkInstance;
    let mut raw_surface: glfw_sys::VkSurfaceKHR = std::ptr::null_mut();

    let result = unsafe {
        glfw_sys::glfwCreateWindowSurface(
            raw_instance,
            glfw_window,
            std::ptr::null(),
            &mut raw_surface as *mut glfw_sys::VkSurfaceKHR,
        )
    };

    if result != 0 {
        return Err(format!("glfwCreateWindowSurface failed with code {result}"));
    }
    if raw_surface.is_null() {
        return Err("glfwCreateWindowSurface succeeded but returned a null surface".to_string());
    }

    Ok(vk::SurfaceKHR::from_raw(raw_surface as usize as u64))
}

fn allocate_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    count: u32,
) -> Result<Vec<vk::CommandBuffer>, String> {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);

    unsafe {
        device
            .allocate_command_buffers(&alloc_info)
            .map_err(|e| format!("vkAllocateCommandBuffers failed: {e:?}"))
    }
}

fn create_swapchain(
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    swapchain_loader: &ash::khr::swapchain::Device,
    window: &Window,
    queue_family_index: u32,
) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D), String> {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface)
            .map_err(|e| format!("vkGetPhysicalDeviceSurfaceCapabilitiesKHR failed: {e:?}"))?
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(physical_device, surface)
            .map_err(|e| format!("vkGetPhysicalDeviceSurfaceFormatsKHR failed: {e:?}"))?
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface)
            .map_err(|e| format!("vkGetPhysicalDeviceSurfacePresentModesKHR failed: {e:?}"))?
    };

    let surface_format = formats
        .iter()
        .copied()
        .find(|f| f.format == vk::Format::B8G8R8A8_UNORM)
        .unwrap_or_else(|| formats[0]);

    let present_mode = present_modes
        .into_iter()
        .find(|&m| m == vk::PresentModeKHR::FIFO)
        .unwrap_or(vk::PresentModeKHR::FIFO);

    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let (fb_w, fb_h) = window.framebuffer_size();
        let w = (fb_w.max(1) as u32).clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
        let h = (fb_h.max(1) as u32).clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);
        vk::Extent2D { width: w, height: h }
    };

    let mut image_count = capabilities.min_image_count.saturating_add(1);
    if capabilities.max_image_count > 0 {
        image_count = image_count.min(capabilities.max_image_count);
    }

    let indices = [queue_family_index];

    let create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&indices)
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&create_info, None)
            .map_err(|e| format!("vkCreateSwapchainKHR failed: {e:?}"))?
    };

    let images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .map_err(|e| format!("vkGetSwapchainImagesKHR failed: {e:?}"))?
    };

    let _ = device;
    Ok((swapchain, images, surface_format.format, extent))
}

fn create_image_views(device: &ash::Device, images: &[vk::Image], format: vk::Format) -> Result<Vec<vk::ImageView>, String> {
    let mut views = Vec::with_capacity(images.len());
    for &image in images {
        let create = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );
        let view = unsafe { device.create_image_view(&create, None) }
            .map_err(|e| format!("vkCreateImageView failed: {e:?}"))?;
        views.push(view);
    }
    Ok(views)
}

fn create_render_pass(device: &ash::Device, format: vk::Format) -> Result<vk::RenderPass, String> {
    let color_attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_ref));

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let info = vk::RenderPassCreateInfo::default()
        .attachments(std::slice::from_ref(&color_attachment))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dependency));

    unsafe { device.create_render_pass(&info, None) }
        .map_err(|e| format!("vkCreateRenderPass failed: {e:?}"))
}

fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    views: &[vk::ImageView],
    extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>, String> {
    let mut fbs = Vec::with_capacity(views.len());

    for &view in views {
        let attachments = [view];
        let info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let fb = unsafe { device.create_framebuffer(&info, None) }
            .map_err(|e| format!("vkCreateFramebuffer failed: {e:?}"))?;
        fbs.push(fb);
    }

    Ok(fbs)
}
