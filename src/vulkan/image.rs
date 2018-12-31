  fn create_texture_sampler(vk: &vk::DevicePointers, device: &vk::Device) -> vk::Sampler {
    let mut sampler: vk::Sampler = unsafe { mem::uninitialized() };
    
    let mag_filter = vk::FILTER_NEAREST;
    let min_filter = vk::FILTER_NEAREST;
    let mipmap_mode = vk::SAMPLER_MIPMAP_MODE_LINEAR;
    let address_mode = vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    
    let sampler_create_info = {
      vk::SamplerCreateInfo {
        sType: vk::STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        magFilter: mag_filter,
        minFilter: min_filter,
        mipmapMode: mipmap_mode,
        addressModeU: address_mode,
        addressModeV: address_mode,
        addressModeW: address_mode,
        mipLodBias: 0.0,
        anisotropyEnable: vk::TRUE,
        maxAnisotropy: 16.0,
        compareEnable: vk::FALSE,
        compareOp: vk::COMPARE_OP_ALWAYS,
        minLod: 0.0,
        maxLod: 0.0,
        borderColor: vk::BORDER_COLOR_INT_OPAQUE_BLACK,
        unnormalizedCoordinates: vk::FALSE,
      }
    };
    
    unsafe {
      check_errors(vk.CreateSampler(*device, &sampler_create_info, ptr::null(), &mut sampler));
    }
    
    sampler
  }
  
  fn create_image_view(vk: &vk::DevicePointers, device: &vk::Device, image: &vk::Image, format: &vk::Format) -> vk::ImageView {
    let mut image_view: vk::ImageView = unsafe { mem::uninitialized() };
    
    let component = vk::ComponentMapping {
      r: vk::COMPONENT_SWIZZLE_IDENTITY,
      g: vk::COMPONENT_SWIZZLE_IDENTITY,
      b: vk::COMPONENT_SWIZZLE_IDENTITY,
      a: vk::COMPONENT_SWIZZLE_IDENTITY,
    };
    
    let subresource = vk::ImageSubresourceRange {
      aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
      baseMipLevel: 0,
      levelCount: 1,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let image_view_create_info = vk::ImageViewCreateInfo {
      sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      image: *image,
      viewType: vk::IMAGE_VIEW_TYPE_2D,
      format: *format,
      components: component,
      subresourceRange: subresource,
    };
    
    unsafe {
      vk.CreateImageView(*device, &image_view_create_info, ptr::null(), &mut image_view);
    }
    
    image_view
  }
  
  fn create_texture_image(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, swapchain_format: &vk::Format, location: String) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let image = image::open(&location.clone()).expect(&("No file or Directory at: ".to_string() + &location)).to_rgba(); 
    let (width, height) = image.dimensions();
    let image_data = image.into_raw().clone();
    
    let image_size: vk::DeviceSize = (width * height * 4).into();
    
    let mut texture_image: vk::Image = unsafe { mem::uninitialized() };
    let mut texture_memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    let mut texture_image_view: vk::ImageView;
    
    Vulkan::create_image(vk, vk_instance, device, phys_device, vk::Extent2D { width: width, height: height }, swapchain_format, vk::IMAGE_TILING_OPTIMAL, vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT, &mut texture_image, &mut texture_memory);
    
    texture_image_view = Vulkan::create_image_view(vk, device, &texture_image, swapchain_format);
    
    (texture_image, texture_memory, texture_image_view)
  }
  
  fn create_image(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, image_extent: vk::Extent2D, format: &vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags, image: &mut vk::Image, image_memory: &mut vk::DeviceMemory) {
    //
    // Start Create image
    //
    let image_create_info = {
      vk::ImageCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        imageType: vk::IMAGE_TYPE_2D,
        format: *format,
        extent: vk::Extent3D { width: image_extent.width, height: image_extent.height, depth: 1 },
        mipLevels: 1,
        arrayLayers: 1,
        samples: vk::SAMPLE_COUNT_1_BIT,
        tiling: tiling,
        usage: usage,
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
        initialLayout: vk::IMAGE_LAYOUT_PREINITIALIZED,
      }
    };
    
   let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::uninitialized() };
    
    unsafe {
      check_errors(vk.CreateImage(*device, &image_create_info, ptr::null(), image));
      vk.GetImageMemoryRequirements(*device, *image, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        vk_instance.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      for i in 0..memory_properties.memoryTypeCount as usize {
        if memory_requirements.memoryTypeBits & (1 << i) != 0 && memory_properties.memoryTypes[i].propertyFlags & properties == properties {
          index = i as i32;
        }
      }
      
      if index == -1 {
        panic!("Failed to find suitable memory type");
      }
      
      index
    };
    
    let memory_allocate_info = {
      vk::MemoryAllocateInfo {
        sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: ptr::null(),
        allocationSize: memory_requirements.size,
        memoryTypeIndex: memory_type_bits_index as u32,
      }
    };
    
    unsafe {
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), image_memory));
      check_errors(vk.BindImageMemory(*device, *image, *image_memory, 0));
    }
  }
  
