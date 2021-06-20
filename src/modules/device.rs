use ash::extensions::{
  khr::{Surface, Swapchain},
};

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device};
use std::default::Default;

use std::ffi::{CStr, CString};

use crate::modules::{VkInstance, VkWindow};

/*
 if (data->properties.limits.nonCoherentAtomSize > 0) {
   VkDeviceSize atom_size = data->properties.limits.nonCoherentAtomSize - 1;
   new_size = (new_size + atom_size) & ~atom_size;
 }
*/

pub struct VkDevice {
  device: Device,
  phys_device: vk::PhysicalDevice,
  device_memory_properties: vk::PhysicalDeviceMemoryProperties,
  surface: vk::SurfaceKHR,
  surface_format: vk::SurfaceFormatKHR,
  surface_loader: Surface,
  queue_family_index: u32,
  present_queue: vk::Queue,
}

impl VkDevice {
  pub fn new(instance: &VkInstance, window: &VkWindow) -> VkDevice {
    let surface_loader = Surface::new(instance.entry(), instance);
    let surface = unsafe {
        ash_window::create_surface(instance.entry(), instance, window.internal(), None).unwrap()
    };
    
    let (phys_device, queue_family_index) = pick_physical_device(instance, &surface, &surface_loader);
    let (device, present_queue) = create_logical_device(instance, &phys_device, queue_family_index);
    
    let surface_format = unsafe {
        surface_loader
            .get_physical_device_surface_formats(phys_device, surface)
            .unwrap()[0]
    };
    
    let device_memory_properties = unsafe {
        instance.internal().get_physical_device_memory_properties(phys_device)
    };
    
    VkDevice {
      device,
      phys_device,
      device_memory_properties,
      surface,
      surface_format,
      surface_loader,
      queue_family_index,
      present_queue,
    }
  }
  
  pub fn internal(&self) -> &Device {
    &self.device
  }
  
  pub fn device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
    self.device_memory_properties
  }
  
  pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
    self.surface_format
  }
  
  pub fn phys_device(&self) -> &vk::PhysicalDevice {
    &self.phys_device
  }
  
  pub fn surface(&self) -> &vk::SurfaceKHR {
    &self.surface
  }
  
  pub fn surface_loader(&self) -> &Surface {
    &self.surface_loader
  }
  
  pub fn queue_family_index(&self) -> u32 {
    self.queue_family_index
  }
  
  pub fn present_queue(&self) -> vk::Queue {
    self.present_queue
  }
}

fn pick_physical_device(instance: &VkInstance, surface: &vk::SurfaceKHR, surface_loader: &Surface)
    -> (vk::PhysicalDevice, u32) {
  let pdevices = unsafe {
    instance.internal()
      .enumerate_physical_devices()
      .expect("Physical device error")
  };
  let (pdevice, queue_family_index) = pdevices
    .iter()
    .map(|pdevice| {
      unsafe {
        instance.internal()
          .get_physical_device_queue_family_properties(*pdevice)
          .iter()
          .enumerate()
          .filter_map(|(index, ref info)| {
            let supports_graphic_and_surface =
              info.queue_flags.contains(vk::QueueFlags::GRAPHICS) && 
              info.queue_flags.contains(vk::QueueFlags::COMPUTE) && 
              surface_loader.get_physical_device_surface_support(
                *pdevice,
                index as u32,
                *surface,
              ).unwrap();
            if supports_graphic_and_surface {
              Some((*pdevice, index))
            } else {
              None
            }
          })
          .next()
      }
    })
    .filter_map(|v| v)
    .next()
    .expect("Couldn't find suitable device.");

  (pdevice, queue_family_index as u32)
}

fn create_logical_device(instance: &VkInstance, pdevice: &vk::PhysicalDevice, queue_family_index: u32)
    -> (Device, vk::Queue) {
  
  let priorities = [1.0];
  let queue_info = [vk::DeviceQueueCreateInfo::builder()
                      .queue_family_index(queue_family_index)
                      .queue_priorities(&priorities)
                      .build()];
  let device_extension_names_raw = [Swapchain::name().as_ptr()];
  let features = vk::PhysicalDeviceFeatures {
    shader_clip_distance: 1,
    ..Default::default()
  };
  let device_create_info = vk::DeviceCreateInfo::builder()
                              .queue_create_infos(&queue_info)
                              .enabled_extension_names(&device_extension_names_raw)
                              .enabled_features(&features);

  let device: Device = unsafe {
    instance.internal().create_device(*pdevice, &device_create_info, None).unwrap()
  };

  let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

  (device, present_queue)
}

impl DeviceV1_0 for VkDevice {
  fn handle(&self) -> vk::Device {
    self.device.handle()
  }
  
  fn fp_v1_0(&self) -> &vk::DeviceFnV1_0 {
    self.device.fp_v1_0()
  }
}













