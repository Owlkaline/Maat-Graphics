#[macro_use]
extern crate lazy_static;
extern crate winit;
extern crate shared_library;
#[macro_use]
extern crate vk_sys as vk;
use std::ptr;
use std::mem;
use std::sync::Arc;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_void;

use std::ops::Deref;
use std::os::raw::c_char;
use std::borrow::Borrow;

use winit::dpi::LogicalSize;

use modules::VkWindow;

mod loader;
mod modules;
mod ownage;

const ENGINE_VERSION: u32 = (0 as u32) << 22 | (5 as u32) << 12 | (0 as u32);

fn main() {
  VkWindow::new("TestApplication".to_string(), 0, 1280.0, 720.0, true);
  /*
  
  let device: vk::Device = unsafe {
    let mut device: vk::Device = unsafe { mem::uninitialized() };
    
    let mut physical_device_count = 0;
    check_errors(vk_instance.EnumeratePhysicalDevices(instance, &mut physical_device_count, ptr::null_mut()));
    println!("Number of usable GPUs: {}", physical_device_count);
    
    let mut physical_devices: Vec<vk::PhysicalDevice> = Vec::with_capacity(physical_device_count as usize);
    
    check_errors(vk_instance.EnumeratePhysicalDevices(instance, &mut physical_device_count, physical_devices.as_mut_ptr()));
    physical_devices.set_len(physical_device_count as usize);
    for i in 0..physical_devices.len() as usize {
      let mut device_prop: vk::PhysicalDeviceProperties = unsafe { mem::uninitialized() };
      
      vk_instance.GetPhysicalDeviceProperties(physical_devices[i], &mut device_prop);
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
      vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, ptr::null_mut());
      let mut family_properties = Vec::with_capacity(family_count as usize);
      vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, family_properties.as_mut_ptr());
      family_properties.set_len(family_count as usize);
      
      let mut has_graphics_bit = false;
      let mut device_supports_surface: u32 = 0;
      let mut supported_queue_fam_index = 0;
      
      let mut queue_index = 0;
      for j in 0..family_properties.len() {
        println!("  Queue: {}", j);
        
        let queue_count = family_properties[j].queueCount;
        let mut queue_flags = family_properties[j].queueFlags;
        if queue_flags % 2 != 0 {
          println!("    Graphics: True");
          has_graphics_bit = true;
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
        
        if device_supports_surface == 0 {
          unsafe {
            vk_instance.GetPhysicalDeviceSurfaceSupportKHR(physical_devices[i], j as u32, surface, &mut device_supports_surface);
          }
          if device_supports_surface != 0 {
            supported_queue_fam_index = j;
          }
        }
      }

      
      if has_graphics_bit && device_supports_surface != 0 {
        let mut property_count = 0;
        let mut device_extensions;
        unsafe {
          vk_instance.EnumerateDeviceExtensionProperties(physical_devices[i], ptr::null(), &mut property_count, ptr::null_mut());
          device_extensions = Vec::with_capacity(property_count as usize);
          vk_instance.EnumerateDeviceExtensionProperties(physical_devices[i], ptr::null(), &mut property_count, device_extensions.as_mut_ptr());
          device_extensions.set_len(property_count as usize);
        }
        
        let supported_device_extensions: Vec<CString>
           = device_extensions.iter().map(|x| unsafe { CStr::from_ptr(x.extensionName.as_ptr()) }.to_owned()).collect();
          
          let mut device_available_extensions = Vec::new();
          for supported_device_extension in &supported_device_extensions {
            for available_extension in &available_extensions {
              if available_extension == supported_device_extension {
                device_available_extensions.push(supported_device_extension);
              }
            }
          }
          
          
        let device_available_extensions_raw: Vec<*const i8> = device_available_extensions.iter().map(|raw_name| raw_name.as_ptr()).collect();
        
        let mut device_queue_infos = Vec::with_capacity(family_properties.len());
        for j in 0..family_properties.len() {
          let queue_count = family_properties[j].queueCount;
          let queue_flags = family_properties[j].queueFlags;
          device_queue_infos.push( 
            vk::DeviceQueueCreateInfo {
              sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
              pNext: ptr::null(),
              flags: Default::default(),//queue_flags,
              queueFamilyIndex: j as u32,
              queueCount: family_count-1,
              pQueuePriorities: &1.0,
            }
          );
        }
        
        let device_info = vk::DeviceCreateInfo {
          sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
          pNext: ptr::null(),
          flags: Default::default(),
          queueCreateInfoCount: family_properties.len() as u32,
          pQueueCreateInfos: device_queue_infos.as_ptr(),
          ppEnabledLayerNames: layers_names_raw.as_ptr(),
          enabledLayerCount: layers_names_raw.len() as u32,
          ppEnabledExtensionNames: device_available_extensions_raw.as_ptr(),
          enabledExtensionCount: device_available_extensions_raw.len() as u32,
          pEnabledFeatures: ptr::null(), // For more features use vk::GetPhysicalDeviceFeatures
        };
        
        check_errors(vk_instance.CreateDevice(physical_devices[i], &device_info, ptr::null(), &mut device));
        break;
      }
    }
    
    device
  };
  /*
  let surface_capabilities = check_errors(vk_instance.GetPhysicalDeviceSurfaceCapabilitiesKHR());
  
  let swapchain_info = vk::SwapchainCreateInfoKHR {
    sType: vk::STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
    pNext: ptr::null(),
    flags: ptr::null(),
    surface: surface,
    minImageCount: 
    imageFormat: 
    imageColorSpace: 
    imageFormat: 
    imageExtent: 
    imageArrayLayers: 
    imageUsage: 
    imageSharingMode: 
    queueFamilyIndexCount: 
    pQueueFamilyIndices: 
    preTransform: 
    compositeAlpha: 
    presentMode: 
    clipped: 
    oldSwapchain: 
  };
  
  let mut swapchain;
  check_errors(vk_device.CreateSwapchainKHR(&device, , ptr::null(), swapchain.as_mut_ptr()))
  */
  let vk_device = vk::DevicePointers::load(|name| unsafe {
     vk_instance.GetDeviceProcAddr(device, name.as_ptr()) as *const _
  });
  
  unsafe {
    vk_device.DeviceWaitIdle(device);
    vk_device.DestroyDevice(device, ptr::null());
    vk_instance.DestroyInstance(instance, ptr::null());
  }*/
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_test() {
    assert_eq!(4, 2+2);
  }
}
