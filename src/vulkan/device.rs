use vk;

use crate::vulkan::Instance;

use std::mem;
use std::ptr;
use std::sync::Arc;
use std::ffi::CStr;
use std::ffi::CString;

pub struct Device {
  vk: vk::DevicePointers,
  device: vk::Device,
  phys_device: vk::PhysicalDevice,
  _extensions: Vec<CString>,
}

impl Device {
  pub fn new(instance: Arc<Instance>, surface: &vk::SurfaceKHR) -> Arc<Device> {
    let (device, phys_device, extensions) = Device::create_suitable_device(Arc::clone(&instance), surface);
    let vk = Device::create_device_instance(Arc::clone(&instance), &device);
    
    Arc::new(Device {
      vk: vk,
      device: device,
      phys_device: phys_device,
      _extensions: extensions,
    })
  }
  
  pub fn pointers(&self) -> &vk::DevicePointers {
    &self.vk
  }
  
  pub fn internal_object(&self) -> &vk::Device {
    &self.device
  }
  
  pub fn physical_device(&self) -> &vk::PhysicalDevice {
    &self.phys_device
  }
  
  pub fn get_device_queue(&self, family: u32, index: u32) -> vk::Queue {
    let mut graphics_queue: vk::Queue = unsafe { mem::uninitialized() };
    
    unsafe {
      self.vk.GetDeviceQueue(self.device, family, index, &mut graphics_queue);
    }
    
    graphics_queue
  }
  
  pub fn wait(&self) {
    unsafe {
      println!("Waiting for device to idle");
      self.vk.DeviceWaitIdle(self.device);
    }
  }
  
  pub fn destroy(&self) {
    unsafe {
      self.vk.DestroyDevice(self.device, ptr::null());
    }
  }
  
  fn create_device_instance(instance: Arc<Instance>, device: &vk::Device) -> vk::DevicePointers {
    let vk = instance.pointers();
    
    let vk_device = vk::DevicePointers::load(|name| unsafe {
      vk.GetDeviceProcAddr(*device, name.as_ptr()) as *const _
    });
    
    vk_device
  }
  
  fn create_suitable_device(instance: Arc<Instance>, surface: &vk::SurfaceKHR) -> (vk::Device, vk::PhysicalDevice, Vec<CString>) {
    let layer_names = instance.get_layers();
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let physical_devices = instance.enumerate_physical_devices();
    
    Device::print_physical_device_details(instance.pointers(), &physical_devices);
    
    let mut device: vk::Device = unsafe { mem::uninitialized() };
    let mut device_available_extensions = Vec::new();
    let mut physical_device_index = 0;
    
    for i in 0..physical_devices.len() {
      let family_properties = instance.get_device_queue_family_properties(&physical_devices[i]);
      
      let mut has_graphics_bit = false;
      let mut device_supports_surface: u32 = 0;
      //let mut supported_queue_fam_index = 0;
      
      //let mut queue_index = 0;
      for j in 0..family_properties.len() {
        //let queue_count = family_properties[j].queueCount;
        let queue_flags = family_properties[j].queueFlags;
        if Device::has_graphics_bit(&queue_flags) {
          has_graphics_bit = true;
        }
        
        if device_supports_surface == 0 {
          device_supports_surface = instance.physical_device_supports_surface(&physical_devices[i], j as u32, surface);
          
          //if device_supports_surface != 0 {
          // supported_queue_fam_index = j;
          //}
        }
      }
      
      if has_graphics_bit && device_supports_surface != 0 {
        let device_extensions = instance.enumerate_device_extension_properties(&physical_devices[i]);
        
        let mut available_extensions = instance.get_extensions();
        available_extensions.push(CString::new("VK_KHR_swapchain").unwrap());
        available_extensions.push(CString::new("VK_KHR_display_swapchain").unwrap());
        //available_extensions.push(CString::new("VK_KHR_sampler_mirror_clamp_to_edge").unwrap());
        // available_extensions.push(CString::new("VK_KHR_get_memory_requirements2").unwrap());
        //available_extensions.push(CString::new("VK_KHR_dedicated_allocation").unwrap());
        //available_extensions.push(CString::new("VK_KHR_incremental_present").unwrap());
        available_extensions.push(CString::new("VK_EXT_debug_markers").unwrap());
        
        let supported_device_extensions: Vec<CString>
           = device_extensions.iter().map(|x| unsafe { CStr::from_ptr(x.extensionName.as_ptr()) }.to_owned()).collect();
          
          for supported_device_extension in supported_device_extensions {
            for available_extension in &available_extensions {
              if *available_extension == supported_device_extension {
                device_available_extensions.push(supported_device_extension.clone());
              }
            }
          }
          
        let device_available_extensions_raw: Vec<*const i8> = device_available_extensions.iter().map(|raw_name| raw_name.as_ptr()).collect();
        
        let mut device_queue_infos = Vec::with_capacity(family_properties.len());
        for j in 0..family_properties.len() {
          device_queue_infos.push( 
            vk::DeviceQueueCreateInfo {
              sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
              pNext: ptr::null(),
              flags: 0,//Default::default(),//queue_flags,
              queueFamilyIndex: j as u32,
              queueCount: family_properties.len().min(j.max(1)) as u32,
              pQueuePriorities: &1.0,
            }
          );
        }
        
        let features: vk::PhysicalDeviceFeatures = instance.get_device_features(&physical_devices[physical_device_index]);
        
        match features.shaderSampledImageArrayDynamicIndexing {
          vk::TRUE => {
            println!("Dynamic indexing supported!");
          },
          _ => {println!("Dynamic indexing not supported :(");}
        }
        
        //features.robustBufferAccess = vk::TRUE;
        
        let device_info = vk::DeviceCreateInfo {
          sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
          pNext: ptr::null(),
          flags: 0,
          queueCreateInfoCount: family_properties.len() as u32,
          pQueueCreateInfos: device_queue_infos.as_ptr(),
          ppEnabledLayerNames: layers_names_raw.as_ptr(),
          enabledLayerCount: layers_names_raw.len() as u32,
          ppEnabledExtensionNames: device_available_extensions_raw.as_ptr(),
          enabledExtensionCount: device_available_extensions_raw.len() as u32,
          pEnabledFeatures: &features, // For more features use vk::GetPhysicalDeviceFeatures
        };
        
        device = instance.create_device(&physical_devices[i], &device_info);
        
        physical_device_index = i;
        break;
      }
    }
    
    (device, physical_devices[physical_device_index], device_available_extensions)
  }
  
  fn print_physical_device_details(vk_instance: &vk::InstancePointers, physical_devices: &Vec<vk::PhysicalDevice>) {
    for i in 0..physical_devices.len() as usize {
      let mut device_prop: vk::PhysicalDeviceProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        vk_instance.GetPhysicalDeviceProperties(physical_devices[i], &mut device_prop);
      }
      
      println!("min alignment: {}", device_prop.limits.minUniformBufferOffsetAlignment);
      println!("max push constant size: {}", device_prop.limits.maxPushConstantsSize);
      let device_name = device_prop.deviceName.iter().map(|a| { 
        let mut b = (*a as u8 as char).to_string();
        if b == "\u{0}".to_string() {
          b = "".to_string();
        }
        b
      }).collect::<String>();
      
      let device_type = device_prop.deviceType;
      let mut device_type_name = "";
      
      match device_type {
       vk::PHYSICAL_DEVICE_TYPE_OTHER => { device_type_name = "Other GPU"; },
       vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU => { device_type_name = "Integrated GPU"; },
       vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU => { device_type_name = "Discrete GPU"; },
       vk::PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU => { device_type_name = "Virtual GPU"; },
       vk::PHYSICAL_DEVICE_TYPE_CPU => { device_type_name = "CPU"; },
        _ => {},
      }
      
      println!("{}: {} -> {}", i, device_type_name, device_name);
    }
    
    for i in 0..physical_devices.len() {
      println!("Device: {}", i);
      let mut family_count = 0;
      
      unsafe {
        vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, ptr::null_mut());
      }
      
      let mut family_properties = Vec::with_capacity(family_count as usize);
      
      unsafe {
        vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, family_properties.as_mut_ptr());
        family_properties.set_len(family_count as usize);
      }
      
      //let mut queue_index = 0;
      for j in 0..family_properties.len() {
        println!("  Queue: {}", j);
        let mut queue_flags = family_properties[j].queueFlags;
        if Device::has_graphics_bit(&queue_flags) {
          println!("    Graphics: True");
          queue_flags -= 1;
        } else {
          println!("    Graphics: False");
        };
        if queue_flags >= 8 {
          println!("     Binding: True");
          queue_flags -= 8;
        } else {
          println!("     Binding: False");
        }
        if queue_flags >= 4 {
          println!("    Transfer: True");
          queue_flags -= 4;
        } else {
          println!("    Transfer: False");
        }
        if queue_flags != 0 {
          println!("     Compute: True");
        } else {
          println!("     Compute: False");
        }
      }
    }
  }
  
  fn has_graphics_bit(queue_flags: &u32) -> bool {
    queue_flags % 2 != 0 
  }
}
