use std::{
    cell::RefCell,
    ffi::CString,
    os::raw::c_char,
    sync::Arc,
};
use std::time::{Duration, Instant};

use ash::{vk, Entry};
use ash::vk::Handle;

use super::{vulkan_buffer, vulkan_pipeline, VulkanBuffer, VulkanPipeline};

use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
use crate::window::Window;

use super::VulkanShader;

pub(crate) struct VulkanGraphics {
    state: RefCell<VulkanState>,
}

struct VulkanState {
    #[allow(dead_code)]
    entry: Entry,
    instance: ash::Instance,
    surface_loader: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,

    physical_device: vk::PhysicalDevice,

    device: ash::Device,
    queue_family_index: u32,
    queue: vk::Queue,

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

    in_frame: bool,
    current_image_index: u32,
    current_cmd: vk::CommandBuffer,

    viewport: Option<vk::Viewport>,
    scissor: Option<vk::Rect2D>,

    swapchain_dirty: bool,
    surface_lost: bool,
}

impl VulkanGraphics {
    pub(crate) fn create(window: &mut Window) -> Result<Self, String> {
        let entry = unsafe { Entry::load().map_err(|e| format!("Failed to load Vulkan entry: {e:?}"))? };

        let app_name = CString::new("kingogfx").unwrap();
        let engine_name = CString::new("kingogfx").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_0);

        let required_extensions = glfw_required_instance_extensions()?;

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&required_extensions);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| format!("vkCreateInstance failed: {e:?}"))?
        };

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let surface = create_surface(&instance, window)?;

        let (physical_device, queue_family_index) = pick_physical_device(&instance, &surface_loader, surface)?;

        let queue_priorities = [1.0f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extensions);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| format!("vkCreateDevice failed: {e:?}"))?
        };

        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let (swapchain, swapchain_images, swapchain_format, swapchain_extent) = create_swapchain(
            physical_device,
            &device,
            &surface_loader,
            surface,
            &swapchain_loader,
            window,
            queue_family_index,
        )?;

        let swapchain_image_views = create_image_views(&device, &swapchain_images, swapchain_format)?;
        let render_pass = create_render_pass(&device, swapchain_format)?;
        let framebuffers = create_framebuffers(&device, render_pass, &swapchain_image_views, swapchain_extent)?;

        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .map_err(|e| format!("vkCreateCommandPool failed: {e:?}"))?
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(swapchain_images.len() as u32);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| format!("vkAllocateCommandBuffers failed: {e:?}"))?
        };

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
            state: RefCell::new(VulkanState {
                entry,
                instance,
                surface_loader,
                surface,
                physical_device,
                device,
                queue_family_index,
                queue,
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
                in_frame: false,
                current_image_index: 0,
                current_cmd: vk::CommandBuffer::null(),
                viewport: None,
                scissor: None,
                swapchain_dirty: false,
                surface_lost: false,
            }),
        })
    }

    pub(crate) fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let mut state = self.state.borrow_mut();
        state.viewport = Some(vk::Viewport {
            x: x as f32,
            y: y as f32,
            width: width.max(0) as f32,
            height: height.max(0) as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        state.scissor = Some(vk::Rect2D {
            offset: vk::Offset2D { x, y },
            extent: vk::Extent2D {
                width: width.max(0) as u32,
                height: height.max(0) as u32,
            },
        });
    }

    fn recreate_swapchain(&mut self, window: &mut Window, recreate_surface: bool) -> Result<(), String> {
        let mut state = self.state.borrow_mut();

        let old_format = state.swapchain_format;

        // If minimized, don't try to recreate. Let caller retry later.
        let (fb_w, fb_h) = window.framebuffer_size();
        if fb_w <= 0 || fb_h <= 0 {
            return Err("Framebuffer size is 0 (minimized)".to_string());
        }

        unsafe {
            // Best-effort wait for the previous frame.
            let _ = state.device.wait_for_fences(&[state.in_flight], true, 2_000_000_000);
        }

        unsafe {
            for fb in &state.framebuffers {
                state.device.destroy_framebuffer(*fb, None);
            }
            state.framebuffers.clear();

            for view in &state.swapchain_image_views {
                state.device.destroy_image_view(*view, None);
            }
            state.swapchain_image_views.clear();

            if !state.command_buffers.is_empty() {
                state.device.free_command_buffers(state.command_pool, &state.command_buffers);
                state.command_buffers.clear();
            }

            if state.swapchain != vk::SwapchainKHR::null() {
                state.swapchain_loader.destroy_swapchain(state.swapchain, None);
                state.swapchain = vk::SwapchainKHR::null();
            }

            if recreate_surface {
                if state.surface != vk::SurfaceKHR::null() {
                    state.surface_loader.destroy_surface(state.surface, None);
                    state.surface = vk::SurfaceKHR::null();
                }
                state.surface = create_surface(&state.instance, window)?;
            }
        }

        let (swapchain, images, format, extent) = create_swapchain(
            state.physical_device,
            &state.device,
            &state.surface_loader,
            state.surface,
            &state.swapchain_loader,
            window,
            state.queue_family_index,
        )?;

        state.swapchain = swapchain;
        state.swapchain_images = images;
        state.swapchain_format = format;
        state.swapchain_extent = extent;

        state.swapchain_image_views = create_image_views(&state.device, &state.swapchain_images, state.swapchain_format)?;

        // Only recreate the render pass if the swapchain format changed.
        // Keeping the render pass stable allows pipelines created once at startup
        // to remain valid across resizes.
        if state.render_pass == vk::RenderPass::null() || state.swapchain_format != old_format {
            unsafe {
                if state.render_pass != vk::RenderPass::null() {
                    state.device.destroy_render_pass(state.render_pass, None);
                }
            }
            state.render_pass = create_render_pass(&state.device, state.swapchain_format)?;
        }

        state.framebuffers = create_framebuffers(
            &state.device,
            state.render_pass,
            &state.swapchain_image_views,
            state.swapchain_extent,
        )?;

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(state.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(state.swapchain_images.len() as u32);

        state.command_buffers = unsafe {
            state
                .device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| format!("vkAllocateCommandBuffers failed: {e:?}"))?
        };

        state.viewport = None;
        state.scissor = None;
        state.swapchain_dirty = false;
        state.surface_lost = false;
        state.in_frame = false;
        state.current_cmd = vk::CommandBuffer::null();
        state.current_image_index = 0;
        Ok(())
    }

    fn recreate_swapchain_if_needed(&mut self, window: &mut Window) -> Result<(), String> {
        let (fb_w, fb_h) = window.framebuffer_size();
        if fb_w <= 0 || fb_h <= 0 {
            return Err("Framebuffer size is 0 (minimized)".to_string());
        }

        let (dirty, surface_lost, extent) = {
            let state = self.state.borrow();
            (state.swapchain_dirty, state.surface_lost, state.swapchain_extent)
        };

        if dirty || extent.width != fb_w as u32 || extent.height != fb_h as u32 {
            self.recreate_swapchain(window, surface_lost)?;
        }

        Ok(())
    }

    pub(crate) fn create_buffer_init(&mut self, data: &[f32], usage: BufferUsage) -> Result<VulkanBuffer, String> {
        let state = self.state.borrow();
        vulkan_buffer::create_buffer_init(
            &state.instance,
            state.physical_device,
            &state.device,
            data,
            usage,
        )
    }

    pub(crate) fn create_shader(&mut self, desc: ShaderDescriptor<'_>) -> Result<Arc<VulkanShader>, String> {
        let state = self.state.borrow();
        let shader = VulkanShader::from_glsl_sources(
            &state.device,
            desc.vertex_source_glsl,
            desc.fragment_source_glsl,
        )?;
        Ok(Arc::new(shader))
    }

    pub(crate) fn create_pipeline(&mut self, shader: &Arc<VulkanShader>) -> Result<VulkanPipeline, String> {
        let state = self.state.borrow();
        vulkan_pipeline::create_pipeline(&state.device, state.render_pass, shader)
    }

    pub(crate) fn begin_frame(&mut self, _window: &mut Window, clear: ClearColor) -> Result<(), String> {
        {
            let state = self.state.borrow();
            if state.in_frame {
                return Err("begin_frame called while already in a frame".to_string());
            }
        }

        // Handle resize/suboptimal/out-of-date before acquire.
        self.recreate_swapchain_if_needed(_window)?;

        // Wait for the previous frame to complete.
        {
            let state = self.state.borrow();
            unsafe {
                state
                    .device
                    .wait_for_fences(&[state.in_flight], true, 5_000_000_000)
                    .map_err(|e| {
                        if e == vk::Result::TIMEOUT {
                            "vkWaitForFences timed out (GPU may be hung)".to_string()
                        } else {
                            format!("vkWaitForFences failed: {e:?}")
                        }
                    })?;
            }
        }

        // Acquire next image with at most one swapchain recreation retry.
        let mut attempts_left = 2;
        let (image_index, suboptimal) = loop {
            let acquire_result = {
                let state = self.state.borrow();
                unsafe {
                    state
                        .swapchain_loader
                        .acquire_next_image(state.swapchain, u64::MAX, state.image_available, vk::Fence::null())
                }
            };

            match acquire_result {
                Ok(v) => break v,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    {
                        let mut state = self.state.borrow_mut();
                        state.swapchain_dirty = true;
                    }
                    attempts_left -= 1;
                    if attempts_left == 0 {
                        return Err("vkAcquireNextImageKHR out-of-date".to_string());
                    }
                    self.recreate_swapchain_if_needed(_window)?;
                }
                Err(vk::Result::ERROR_SURFACE_LOST_KHR) => {
                    {
                        let mut state = self.state.borrow_mut();
                        state.swapchain_dirty = true;
                        state.surface_lost = true;
                    }
                    attempts_left -= 1;
                    if attempts_left == 0 {
                        return Err("vkAcquireNextImageKHR surface lost".to_string());
                    }
                    self.recreate_swapchain_if_needed(_window)?;
                }
                Err(e) => return Err(format!("vkAcquireNextImageKHR failed: {e:?}")),
            }
        };

        if suboptimal {
            let mut state = self.state.borrow_mut();
            state.swapchain_dirty = true;
        }

        let mut state = self.state.borrow_mut();
        if state.in_frame {
            return Err("begin_frame called while already in a frame".to_string());
        }

        let cmd = state.command_buffers[image_index as usize];
        unsafe {
            state
                .device
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .map_err(|e| format!("vkResetCommandBuffer failed: {e:?}"))?;
            state
                .device
                .begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default())
                .map_err(|e| format!("vkBeginCommandBuffer failed: {e:?}"))?;
        }

        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [clear.r, clear.g, clear.b, clear.a],
            },
        };

        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: state.swapchain_extent,
        };

        let begin_rp = vk::RenderPassBeginInfo::default()
            .render_pass(state.render_pass)
            .framebuffer(state.framebuffers[image_index as usize])
            .render_area(render_area)
            .clear_values(std::slice::from_ref(&clear_value));

        unsafe {
            state.device.cmd_begin_render_pass(cmd, &begin_rp, vk::SubpassContents::INLINE);

            let viewport = state.viewport.unwrap_or(vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: state.swapchain_extent.width as f32,
                height: state.swapchain_extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            });
            let scissor = state.scissor.unwrap_or(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: state.swapchain_extent,
            });

            state.device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
            state.device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));
        }

        state.in_frame = true;
        state.current_image_index = image_index;
        state.current_cmd = cmd;
        Ok(())
    }

    pub(crate) fn set_pipeline(&mut self, pipeline: &VulkanPipeline) -> Result<(), String> {
        let state = self.state.borrow();
        if !state.in_frame {
            return Err("set_pipeline must be called between begin_frame/end_frame".to_string());
        }
        unsafe {
            state
                .device
                .cmd_bind_pipeline(state.current_cmd, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
        }
        Ok(())
    }

    pub(crate) fn set_vertex_buffer(&mut self, slot: u32, buffer: &VulkanBuffer) -> Result<(), String> {
        let state = self.state.borrow();
        if !state.in_frame {
            return Err("set_vertex_buffer must be called between begin_frame/end_frame".to_string());
        }
        if slot != 0 {
            return Err("Vulkan backend currently supports only slot 0".to_string());
        }
        unsafe {
            state.device.cmd_bind_vertex_buffers(state.current_cmd, 0, &[buffer.buffer], &[0]);
        }
        Ok(())
    }

    pub(crate) fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), String> {
        let state = self.state.borrow();
        if !state.in_frame {
            return Err("draw must be called between begin_frame/end_frame".to_string());
        }
        unsafe {
            state.device.cmd_draw(state.current_cmd, vertex_count, 1, first_vertex, 0);
        }
        Ok(())
    }

    pub(crate) fn end_frame(&mut self, _window: &mut Window) -> Result<(), String> {
        let mut state = self.state.borrow_mut();
        if !state.in_frame {
            return Err("end_frame called without begin_frame".to_string());
        }

        unsafe {
            state.device.cmd_end_render_pass(state.current_cmd);
            state
                .device
                .end_command_buffer(state.current_cmd)
                .map_err(|e| format!("vkEndCommandBuffer failed: {e:?}"))?;
        }

        let wait_semaphores = [state.image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [state.render_finished];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&state.current_cmd))
            .signal_semaphores(&signal_semaphores);

        unsafe {
            state
                .device
                .reset_fences(&[state.in_flight])
                .map_err(|e| format!("vkResetFences failed: {e:?}"))?;
            state
                .device
                .queue_submit(state.queue, &[submit_info], state.in_flight)
                .map_err(|e| format!("vkQueueSubmit failed: {e:?}"))?;
        }

        let swapchains = [state.swapchain];
        let image_indices = [state.current_image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            match state
                .swapchain_loader
                .queue_present(state.queue, &present_info)
            {
                Ok(suboptimal) => {
                    if suboptimal {
                        state.swapchain_dirty = true;
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    // Common during resize/minimize.
                    state.swapchain_dirty = true;
                    state.in_frame = false;
                    state.current_cmd = vk::CommandBuffer::null();
                    return Ok(());
                }
                Err(vk::Result::ERROR_SURFACE_LOST_KHR) => {
                    // Common during close or when the underlying surface becomes invalid.
                    state.swapchain_dirty = true;
                    state.surface_lost = true;
                    state.in_frame = false;
                    state.current_cmd = vk::CommandBuffer::null();
                    return Ok(());
                }
                Err(e) => return Err(format!("vkQueuePresentKHR failed: {e:?}")),
            }
        }

        state.in_frame = false;
        state.current_cmd = vk::CommandBuffer::null();
        Ok(())
    }

    pub(crate) fn shutdown(&mut self, window: &mut Window) -> Result<(), String> {
        // If we're mid-frame, try to finish it; if that fails (surface lost/out-of-date),
        // still continue shutdown.
        {
            let in_frame = self.state.borrow().in_frame;
            if in_frame {
                let _ = self.end_frame(window);
            }
        }

        // Vulkan WSI progress can depend on the OS message pump.
        // Pump events for a short bounded period while waiting for the last submitted
        // work to complete.
        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            let _ = window.poll_events();

            let fence_done = {
                let state = self.state.borrow();
                unsafe {
                    match state.device.wait_for_fences(&[state.in_flight], true, 0) {
                        Ok(_) => true,
                        Err(vk::Result::TIMEOUT) => false,
                        Err(_) => false,
                    }
                }
            };

            if fence_done || Instant::now() >= deadline {
                break;
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        Ok(())
    }
}

impl Drop for VulkanGraphics {
    fn drop(&mut self) {
        let state = match self.state.try_borrow_mut() {
            Ok(s) => s,
            Err(_) => return,
        };
        unsafe {
            // Some drivers/WSI paths can hang in vkDeviceWaitIdle during teardown
            // (especially if the window message pump stops). Do a bounded wait on
            // our in-flight fence as a best-effort sync to avoid TDRs.
            let _ = state.device.wait_for_fences(&[state.in_flight], true, 2_000_000_000);

            state.device.destroy_fence(state.in_flight, None);
            state.device.destroy_semaphore(state.render_finished, None);
            state.device.destroy_semaphore(state.image_available, None);

            state.device.free_command_buffers(state.command_pool, &state.command_buffers);
            state.device.destroy_command_pool(state.command_pool, None);

            for fb in &state.framebuffers {
                state.device.destroy_framebuffer(*fb, None);
            }
            state.device.destroy_render_pass(state.render_pass, None);

            for view in &state.swapchain_image_views {
                state.device.destroy_image_view(*view, None);
            }

            state.swapchain_loader.destroy_swapchain(state.swapchain, None);
            state.surface_loader.destroy_surface(state.surface, None);

            state.device.destroy_device(None);
            state.instance.destroy_instance(None);
        }
    }
}

fn glfw_required_instance_extensions() -> Result<Vec<*const c_char>, String> {
    unsafe {
        let mut count: u32 = 0;
        let ptr = glfw_sys::glfwGetRequiredInstanceExtensions(&mut count as *mut u32);
        if ptr.is_null() || count == 0 {
            return Err(
                "GLFW returned no Vulkan instance extensions (is GLFW built with Vulkan support?)".to_string(),
            );
        }
        let slice = std::slice::from_raw_parts(ptr, count as usize);
        Ok(slice.to_vec())
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

fn pick_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32), String> {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .map_err(|e| format!("vkEnumeratePhysicalDevices failed: {e:?}"))?
    };

    for pd in physical_devices {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(pd) };
        for (index, qf) in queue_families.iter().enumerate() {
            let supports_graphics = qf.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            if !supports_graphics {
                continue;
            }
            let supports_present = unsafe {
                surface_loader
                    .get_physical_device_surface_support(pd, index as u32, surface)
                    .map_err(|e| format!("vkGetPhysicalDeviceSurfaceSupportKHR failed: {e:?}"))?
            };
            if supports_present {
                return Ok((pd, index as u32));
            }
        }
    }

    Err("No suitable Vulkan physical device found (need graphics + present support)".to_string())
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
        let info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(std::slice::from_ref(&view))
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        let fb = unsafe { device.create_framebuffer(&info, None) }
            .map_err(|e| format!("vkCreateFramebuffer failed: {e:?}"))?;
        fbs.push(fb);
    }
    Ok(fbs)
}
