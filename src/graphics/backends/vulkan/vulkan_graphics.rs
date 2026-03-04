use std::{
    ffi::CString,
    os::raw::c_char,
    sync::Arc,
};
use std::time::{Duration, Instant};

use ash::{vk, Entry};

use super::{vulkan_buffer, vulkan_pipeline, VulkanBuffer, VulkanPipeline};
use super::vulkan_swapchain::{VulkanSurface, VulkanSwapchain};

use crate::graphics::device::{BufferUsage, ClearColor, ShaderDescriptor};
use crate::window::Window;

use super::VulkanShader;

pub(crate) struct VulkanGraphics {
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,

    device: ash::Device,
    queue_family_index: u32,
    queue: vk::Queue,

    swapchain: VulkanSwapchain,

    in_frame: bool,
    current_image_index: u32,
    current_cmd: vk::CommandBuffer,

    viewport: Option<vk::Viewport>,
    scissor: Option<vk::Rect2D>,
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

        let surface = VulkanSurface::create(&entry, &instance, window)?;

        let (physical_device, queue_family_index) = pick_physical_device(
            &instance,
            surface.surface_loader(),
            surface.surface(),
        )?;

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

        let swapchain = VulkanSwapchain::create(
            &instance,
            physical_device,
            &device,
            surface,
            window,
            queue_family_index,
        )?;

        Ok(Self {
            instance,
            physical_device,
            device,
            queue_family_index,
            queue,
            swapchain,
            in_frame: false,
            current_image_index: 0,
            current_cmd: vk::CommandBuffer::null(),
            viewport: None,
            scissor: None,
        })
    }

    pub(crate) fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.viewport = Some(vk::Viewport {
            x: x as f32,
            y: y as f32,
            width: width.max(0) as f32,
            height: height.max(0) as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        self.scissor = Some(vk::Rect2D {
            offset: vk::Offset2D { x, y },
            extent: vk::Extent2D {
                width: width.max(0) as u32,
                height: height.max(0) as u32,
            },
        });
    }

    fn recreate_swapchain(&mut self, window: &mut Window, recreate_surface: bool) -> Result<(), String> {
        // If minimized, don't try to recreate. Let caller retry later.
        let (fb_w, fb_h) = window.framebuffer_size();
        if fb_w <= 0 || fb_h <= 0 {
            return Err("Framebuffer size is 0 (minimized)".to_string());
        }

        unsafe {
            // Best-effort wait for the previous frame.
            let _ = self.device.wait_for_fences(&[self.swapchain.in_flight_fence()], true, 2_000_000_000);
        }

        self.swapchain.recreate(
            &self.instance,
            self.physical_device,
            &self.device,
            window,
            self.queue_family_index,
            recreate_surface,
        )?;

        self.viewport = None;
        self.scissor = None;
        self.in_frame = false;
        self.current_cmd = vk::CommandBuffer::null();
        self.current_image_index = 0;
        Ok(())
    }

    fn recreate_swapchain_if_needed(&mut self, window: &mut Window) -> Result<(), String> {
        let (fb_w, fb_h) = window.framebuffer_size();
        if fb_w <= 0 || fb_h <= 0 {
            return Err("Framebuffer size is 0 (minimized)".to_string());
        }

        let dirty = self.swapchain.is_dirty();
        let surface_lost = self.swapchain.is_surface_lost();
        let extent = self.swapchain.extent();

        if dirty || extent.width != fb_w as u32 || extent.height != fb_h as u32 {
            self.recreate_swapchain(window, surface_lost)?;
        }

        Ok(())
    }

    pub(crate) fn create_buffer_init(&mut self, data: &[f32], usage: BufferUsage) -> Result<VulkanBuffer, String> {
        vulkan_buffer::create_buffer_init(
            &self.instance,
            self.physical_device,
            &self.device,
            data,
            usage,
        )
    }

    pub(crate) fn create_shader(&mut self, desc: ShaderDescriptor<'_>) -> Result<Arc<VulkanShader>, String> {
        let shader = VulkanShader::from_sources(&self.device, desc.vertex, desc.fragment)?;
        Ok(Arc::new(shader))
    }

    pub(crate) fn create_pipeline(&mut self, shader: &Arc<VulkanShader>) -> Result<VulkanPipeline, String> {
        vulkan_pipeline::create_pipeline(&self.device, self.swapchain.render_pass(), shader)
    }

    pub(crate) fn begin_frame(&mut self, _window: &mut Window, clear: ClearColor) -> Result<(), String> {
        if self.in_frame {
            return Err("begin_frame called while already in a frame".to_string());
        }

        // Handle resize/suboptimal/out-of-date before acquire.
        self.recreate_swapchain_if_needed(_window)?;

        // Wait for the previous frame to complete.
        unsafe {
            self.device
                .wait_for_fences(&[self.swapchain.in_flight_fence()], true, 5_000_000_000)
                .map_err(|e| {
                    if e == vk::Result::TIMEOUT {
                        "vkWaitForFences timed out (GPU may be hung)".to_string()
                    } else {
                        format!("vkWaitForFences failed: {e:?}")
                    }
                })?;
        }

        // Acquire next image with at most one swapchain recreation retry.
        let mut attempts_left = 2;
        let (image_index, suboptimal) = loop {
            let acquire_result = self.swapchain.acquire_next_image();

            match acquire_result {
                Ok(v) => break v,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.swapchain.mark_dirty();
                    attempts_left -= 1;
                    if attempts_left == 0 {
                        return Err("vkAcquireNextImageKHR out-of-date".to_string());
                    }
                    self.recreate_swapchain_if_needed(_window)?;
                }
                Err(vk::Result::ERROR_SURFACE_LOST_KHR) => {
                    self.swapchain.mark_surface_lost();
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
            self.swapchain.mark_dirty();
        }

        if self.in_frame {
            return Err("begin_frame called while already in a frame".to_string());
        }

        let cmd = self.swapchain.command_buffer(image_index);
        unsafe {
            self.device
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .map_err(|e| format!("vkResetCommandBuffer failed: {e:?}"))?;
            self.device
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
            extent: self.swapchain.extent(),
        };

        let begin_rp = vk::RenderPassBeginInfo::default()
            .render_pass(self.swapchain.render_pass())
            .framebuffer(self.swapchain.framebuffer(image_index))
            .render_area(render_area)
            .clear_values(std::slice::from_ref(&clear_value));

        unsafe {
            self.device.cmd_begin_render_pass(cmd, &begin_rp, vk::SubpassContents::INLINE);

            let viewport = self.viewport.unwrap_or(vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent().width as f32,
                height: self.swapchain.extent().height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            });
            let scissor = self.scissor.unwrap_or(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent(),
            });

            self.device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
            self.device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));
        }

        self.in_frame = true;
        self.current_image_index = image_index;
        self.current_cmd = cmd;
        Ok(())
    }

    pub(crate) fn set_pipeline(&mut self, pipeline: &VulkanPipeline) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_pipeline must be called between begin_frame/end_frame".to_string());
        }
        unsafe {
            self.device
                .cmd_bind_pipeline(self.current_cmd, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
        }
        Ok(())
    }

    pub(crate) fn set_vertex_buffer(&mut self, slot: u32, buffer: &VulkanBuffer) -> Result<(), String> {
        if !self.in_frame {
            return Err("set_vertex_buffer must be called between begin_frame/end_frame".to_string());
        }
        if slot != 0 {
            return Err("Vulkan backend currently supports only slot 0".to_string());
        }
        unsafe {
            self.device.cmd_bind_vertex_buffers(self.current_cmd, 0, &[buffer.buffer], &[0]);
        }
        Ok(())
    }

    pub(crate) fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), String> {
        if !self.in_frame {
            return Err("draw must be called between begin_frame/end_frame".to_string());
        }
        unsafe {
            self.device.cmd_draw(self.current_cmd, vertex_count, 1, first_vertex, 0);
        }
        Ok(())
    }

    pub(crate) fn end_frame(&mut self, _window: &mut Window) -> Result<(), String> {
        if !self.in_frame {
            return Err("end_frame called without begin_frame".to_string());
        }

        unsafe {
            self.device.cmd_end_render_pass(self.current_cmd);
            self.device
                .end_command_buffer(self.current_cmd)
                .map_err(|e| format!("vkEndCommandBuffer failed: {e:?}"))?;
        }

        let wait_semaphores = [self.swapchain.image_available()];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.swapchain.render_finished()];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&self.current_cmd))
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device
                .reset_fences(&[self.swapchain.in_flight_fence()])
                .map_err(|e| format!("vkResetFences failed: {e:?}"))?;
            self.device
                .queue_submit(self.queue, &[submit_info], self.swapchain.in_flight_fence())
                .map_err(|e| format!("vkQueueSubmit failed: {e:?}"))?;
        }

        self.swapchain
            .present(self.queue, self.current_image_index)?;

        self.in_frame = false;
        self.current_cmd = vk::CommandBuffer::null();
        Ok(())
    }

    pub(crate) fn shutdown(&mut self, window: &mut Window) -> Result<(), String> {
        // If we're mid-frame, try to finish it; if that fails (surface lost/out-of-date),
        // still continue shutdown.
        if self.in_frame {
            let _ = self.end_frame(window);
        }

        // Vulkan WSI progress can depend on the OS message pump.
        // Pump events for a short bounded period while waiting for the last submitted
        // work to complete.
        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            let _ = window.poll_events();

            let fence_done = {
                unsafe {
                    match self.device.wait_for_fences(&[self.swapchain.in_flight_fence()], true, 0) {
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
        unsafe {
            // Some drivers/WSI paths can hang in vkDeviceWaitIdle during teardown
            // (especially if the window message pump stops). Do a bounded wait on
            // our in-flight fence as a best-effort sync to avoid TDRs.
            let _ = self.device.wait_for_fences(&[self.swapchain.in_flight_fence()], true, 2_000_000_000);

            self.swapchain.destroy(&self.device);

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
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
