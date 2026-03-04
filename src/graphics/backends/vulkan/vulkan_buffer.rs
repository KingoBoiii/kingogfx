use ash::vk;

use crate::graphics::device::BufferUsage;

pub(crate) struct VulkanBuffer {
    pub(super) device: ash::Device,
    pub(super) buffer: vk::Buffer,
    pub(super) memory: vk::DeviceMemory,
}

impl VulkanBuffer {
    pub(crate) fn destroy(&mut self) {
        if self.buffer == vk::Buffer::null() {
            return;
        }
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
        self.buffer = vk::Buffer::null();
        self.memory = vk::DeviceMemory::null();
    }
}

pub(super) fn create_buffer_init(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    data: &[f32],
    usage: BufferUsage,
) -> Result<VulkanBuffer, String> {
    if data.is_empty() {
        return Err("buffer data is empty".to_string());
    }

    let buffer_usage = match usage {
        BufferUsage::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
    };

    let size_bytes = (data.len() * std::mem::size_of::<f32>()) as vk::DeviceSize;
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size_bytes)
        .usage(buffer_usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }
        .map_err(|e| format!("vkCreateBuffer failed: {e:?}"))?;

    let mem_reqs = unsafe { device.get_buffer_memory_requirements(buffer) };
    let mem_type = find_memory_type(
        instance,
        physical_device,
        mem_reqs.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .ok_or_else(|| "No suitable Vulkan memory type found for vertex buffer".to_string())?;

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_reqs.size)
        .memory_type_index(mem_type);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .map_err(|e| format!("vkAllocateMemory failed: {e:?}"))?;

    unsafe {
        device
            .bind_buffer_memory(buffer, memory, 0)
            .map_err(|e| format!("vkBindBufferMemory failed: {e:?}"))?;

        let ptr = device
            .map_memory(memory, 0, size_bytes, vk::MemoryMapFlags::empty())
            .map_err(|e| format!("vkMapMemory failed: {e:?}"))?;
        std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, ptr.cast::<u8>(), size_bytes as usize);
        device.unmap_memory(memory);
    }

    Ok(VulkanBuffer {
        device: device.clone(),
        buffer,
        memory,
    })
}

fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let mem_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    for i in 0..mem_props.memory_type_count {
        let mt = mem_props.memory_types[i as usize];
        if (type_filter & (1 << i)) != 0 && mt.property_flags.contains(properties) {
            return Some(i);
        }
    }
    None
}
