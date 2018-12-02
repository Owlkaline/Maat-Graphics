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
  let app_name = "TestApplication".to_string();
  let vk_window = VkWindow::new(app_name, 0, 1280.0, 720.0, true);
  
  //loop {
    
 // }
  
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
  
  let vk_device = vk::DevicePointers::load(|name| unsafe {
     vk_instance.GetDeviceProcAddr(device, name.as_ptr()) as *const _
  });*/

}

#[cfg(test)]
mod tests {
  #[test]
  fn test_test() {
    assert_eq!(4, 2+2);
  }
}
