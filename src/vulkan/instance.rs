use vk;

use crate::vulkan::ownage::OwnedOrRef;
use crate::vulkan::ownage::check_errors;

use crate::vulkan::loader;
use crate::vulkan::loader::Loader; 
use crate::vulkan::loader::FunctionPointers;

use crate::Logs;

use std::mem;
use std::ptr;
use std::sync::Arc;
use std::ffi::CString;
use std::ffi::CStr;

use crate::ENGINE_VERSION;

fn supported_extensions(entry_points: &vk::EntryPoints) -> Vec<CString> {
  let properties: Vec<vk::ExtensionProperties> = unsafe {
    let mut num = 0;
    check_errors(entry_points.EnumerateInstanceExtensionProperties(
                                   ptr::null(), &mut num, ptr::null_mut()));
    let mut properties = Vec::with_capacity(num as usize);
    check_errors(entry_points.EnumerateInstanceExtensionProperties(
                      ptr::null(), &mut num, properties.as_mut_ptr()));
    properties.set_len(num as usize);
    properties
  };
  
  let supported_extensions: Vec<CString>
   = properties.iter().map(|x| unsafe { CStr::from_ptr(x.extensionName.as_ptr()) }.to_owned()).collect();
   
   supported_extensions
}


pub struct Instance {
  vk: vk::InstancePointers,
  instance: vk::Instance,
  extensions: Vec<CString>,
  layers: Vec<CString>,
}

impl Instance {
  pub fn new(app_name: String, app_version: u32, should_debug: bool) -> Arc<Instance> {
    let function_pointers = OwnedOrRef::Ref(loader::auto_loader().unwrap());
    let entry_points = function_pointers.entry_points();
    let supported_extensions = supported_extensions(entry_points);
    
    let (vk, instance, extensions, layers) = Instance::create_instance(&entry_points, &function_pointers, app_name, app_version, should_debug, supported_extensions);
    
    Arc::new(Instance {
      vk: vk,
      instance: instance,
      extensions: extensions,
      layers: layers,
    })
  }
  
  pub fn pointers(&self) -> &vk::InstancePointers {
    &self.vk
  }
  
  pub fn get_extensions(&self) -> Vec<CString> {
    self.extensions.clone()
  }
  
  pub fn get_layers(&self) -> Vec<CString> {
    self.layers.clone()
  }
  
  pub fn local_instance(&self) -> &vk::Instance {
    &self.instance
  }
  
  pub fn enumerate_physical_devices(&self, logs: &mut Logs) -> Vec<vk::PhysicalDevice> {
    let mut physical_device_count = 0;
    let mut physical_devices: Vec<vk::PhysicalDevice>;
    
    unsafe {
      check_errors(self.vk.EnumeratePhysicalDevices(self.instance, &mut physical_device_count, ptr::null_mut()));
      physical_devices = Vec::with_capacity(physical_device_count as usize);
      check_errors(self.vk.EnumeratePhysicalDevices(self.instance, &mut physical_device_count, physical_devices.as_mut_ptr()));
      physical_devices.set_len(physical_device_count as usize);
    }
    
    logs.system_msg(&format!("Number of usable GPUs: {}", physical_device_count));
    
    physical_devices
  }
  
  pub fn get_queue_family_properties(&self, phys_device: &vk::PhysicalDevice) -> Vec<vk::QueueFamilyProperties> {
    let mut queue_family_properties: Vec<vk::QueueFamilyProperties>;
    
    let mut queue_count = 0;
    
    unsafe {
      self.vk.GetPhysicalDeviceQueueFamilyProperties(*phys_device, &mut queue_count, ptr::null_mut());
      queue_family_properties = Vec::with_capacity(queue_count as usize);
      self.vk.GetPhysicalDeviceQueueFamilyProperties(*phys_device, &mut queue_count, queue_family_properties.as_mut_ptr());
      queue_family_properties.set_len(queue_count as usize);
    }
    
    queue_family_properties
  }
  
  pub fn get_supported_display_queue_families(&self, phys_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR, queue_index: u32) -> u32 {
    let mut present_queues_supported = 0;
    unsafe {
      check_errors(self.vk.GetPhysicalDeviceSurfaceSupportKHR(*phys_device, queue_index, *surface, &mut present_queues_supported));
    }
    
    present_queues_supported
  }
  
  pub fn get_surface_capabilities(&self, phys_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> vk::SurfaceCapabilitiesKHR {
    let mut surface_capabilities: vk::SurfaceCapabilitiesKHR = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    unsafe {
      check_errors(self.vk.GetPhysicalDeviceSurfaceCapabilitiesKHR(*phys_device, *surface, &mut surface_capabilities));
    }
    
    surface_capabilities
  }
  
  pub fn get_physical_device_formats(&self, phys_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> Vec<vk::SurfaceFormatKHR> {
    let mut surface_formats: Vec<vk::SurfaceFormatKHR>;
    let mut num_surface_formats = 0;
    
    unsafe {
      check_errors(self.vk.GetPhysicalDeviceSurfaceFormatsKHR(*phys_device, *surface, &mut num_surface_formats, ptr::null_mut()));
      surface_formats = Vec::with_capacity(num_surface_formats as usize);
      check_errors(self.vk.GetPhysicalDeviceSurfaceFormatsKHR(*phys_device, *surface, &mut num_surface_formats, surface_formats.as_mut_ptr()));
      surface_formats.set_len(num_surface_formats as usize);
    }
    
    surface_formats
  }
  
  pub fn get_present_modes(&self, phys_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> Vec<vk::PresentModeKHR> {
    let mut present_modes: Vec<vk::PresentModeKHR>;
    let mut num_present_modes = 0;
    
    unsafe {
      check_errors(self.vk.GetPhysicalDeviceSurfacePresentModesKHR(*phys_device, *surface, &mut num_present_modes, ptr::null_mut()));
      present_modes = Vec::with_capacity(num_present_modes as usize);
      check_errors(self.vk.GetPhysicalDeviceSurfacePresentModesKHR(*phys_device, *surface, &mut num_present_modes, present_modes.as_mut_ptr()));
      present_modes.set_len(num_present_modes as usize);
    }
    
    present_modes
  }
  
  pub fn get_device_queue_family_properties(&self, phys_device: &vk::PhysicalDevice) -> Vec<vk::QueueFamilyProperties> {
    let mut family_count = 0;
    let mut family_properties;
    unsafe {
      self.vk.GetPhysicalDeviceQueueFamilyProperties(*phys_device, &mut family_count, ptr::null_mut());
      family_properties = Vec::with_capacity(family_count as usize);
      self.vk.GetPhysicalDeviceQueueFamilyProperties(*phys_device, &mut family_count, family_properties.as_mut_ptr());
      family_properties.set_len(family_count as usize);
    }
    
    family_properties
  }
  
  pub fn physical_device_supports_surface(&self, phys_device: &vk::PhysicalDevice, family: u32, surface: &vk::SurfaceKHR) -> u32 {
    let mut device_supports_surface = 0;
    
    unsafe {
      self.vk.GetPhysicalDeviceSurfaceSupportKHR(*phys_device, family, *surface, &mut device_supports_surface);
    }
    
    device_supports_surface
  }
  
  pub fn get_device_properties(&self, phys_device: &vk::PhysicalDevice) -> vk::PhysicalDeviceProperties {
    let mut device_prop: vk::PhysicalDeviceProperties = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    unsafe {
      self.vk.GetPhysicalDeviceProperties(*phys_device, &mut device_prop);
    }
    
    device_prop
  }
  
  pub fn enumerate_device_extension_properties(&self, phys_device: &vk::PhysicalDevice) -> Vec<vk::ExtensionProperties> {
    let mut property_count = 0;
    let mut device_extensions;
    
    unsafe {
      self.vk.EnumerateDeviceExtensionProperties(*phys_device, ptr::null(), &mut property_count, ptr::null_mut());
      device_extensions = Vec::with_capacity(property_count as usize);
      self.vk.EnumerateDeviceExtensionProperties(*phys_device, ptr::null(), &mut property_count, device_extensions.as_mut_ptr());
      device_extensions.set_len(property_count as usize);
    }
    
    device_extensions
  }
  
  pub fn get_device_features(&self, phys_device: &vk::PhysicalDevice) -> vk::PhysicalDeviceFeatures {
    let mut features: vk::PhysicalDeviceFeatures = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    unsafe {
      self.vk.GetPhysicalDeviceFeatures(*phys_device, &mut features);
    }
    
    features
  }
  
  pub fn create_device(&self, phys_device: &vk::PhysicalDevice, device_info: &vk::DeviceCreateInfo) -> vk::Device {
    let mut device = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    unsafe {
      check_errors(self.vk.CreateDevice(*phys_device, device_info, ptr::null(), &mut device));
    }
    
    device
  }
  
  fn create_instance(entry_points: &vk::EntryPoints, function_pointers: &OwnedOrRef<FunctionPointers<Box<dyn Loader + Sync + Send>>>, app_name: String, app_version: u32, should_debug: bool, supported_extensions: Vec<CString>) -> (vk::InstancePointers, vk::Instance, Vec<CString>, Vec<CString>) {
    let app_name = CString::new(app_name).unwrap();
    let engine_name = CString::new("Maat-Graphics").unwrap();
    
    let layer_names = {
      if should_debug {
        [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()]
      } else {
        [CString::new("").unwrap()]
      }
    };
    
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let ideal_extension_names: [CString; 9] = [
      CString::new("VK_KHR_surface").unwrap(),
      CString::new("VK_KHR_xlib_surface").unwrap(),
      CString::new("VK_KHR_xcb_surface").unwrap(),
      CString::new("VK_KHR_wayland_surface").unwrap(),
      CString::new("VK_KHR_android_surface").unwrap(),
      CString::new("VK_KHR_win32_surface").unwrap(),
      CString::new("VK_MVK_ios_surface").unwrap(),
      CString::new("VK_MVK_macos_surface").unwrap(),
      CString::new("VK_EXT_debug_utils").unwrap(),
    ];
    
    let mut available_extensions = Vec::new();
    for supported_extension in &supported_extensions {
      for ideal_extension in &ideal_extension_names {
        if ideal_extension == supported_extension {
          available_extensions.push(supported_extension.clone());
        }
      }
    }
    
    let available_extensions_raw: Vec<*const i8> = available_extensions.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let appinfo = vk::ApplicationInfo {
      pApplicationName: app_name.as_ptr(),
      sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
      pNext: ptr::null(),
      applicationVersion: app_version,
      pEngineName: engine_name.as_ptr(),
      engineVersion: ENGINE_VERSION,
      apiVersion: (1 as u32) << 22 | (0 as u32) << 12 | (5 as u32),
    };
    
    let instance: vk::Instance = unsafe {
      let mut output = mem::MaybeUninit::uninit().assume_init();
      let instance_info = vk::InstanceCreateInfo {
        sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: ptr::null(),
        flags: Default::default(),
        pApplicationInfo: &appinfo,
        ppEnabledLayerNames: if should_debug { layers_names_raw.as_ptr() } else { ptr::null() },
        enabledLayerCount: if should_debug { layers_names_raw.len() as u32 } else { 0 },
        ppEnabledExtensionNames: available_extensions_raw.as_ptr(),
        enabledExtensionCount: available_extensions_raw.len() as u32,
      };
      
      check_errors(entry_points.CreateInstance(&instance_info, ptr::null(), &mut output));
      
      output
    };
    
    let vk_instance = {
      vk::InstancePointers::load(|name| unsafe {
        mem::transmute(function_pointers.get_instance_proc_addr(instance, name.as_ptr()))
      })
    };
    
    (vk_instance, instance, available_extensions, layer_names.to_vec())
  }
  
  fn _has_graphics_bit(queue_flags: &u32) -> bool {
    queue_flags % 2 != 0 
  }
  
  pub fn destroy(&self) {
    println!("Destroying Instance");
    unsafe {
      self.vk.DestroyInstance(self.instance, ptr::null());
    }
  }
}
