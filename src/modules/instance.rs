use ash::extensions::{
    ext::DebugUtils,
};

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Entry, Instance};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

use crate::modules::VkWindow;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

pub struct VkInstance {
  entry: Entry,
  instance: Instance,
  debug_utils_loader: DebugUtils,
  debug_call_back: vk::DebugUtilsMessengerEXT,
}

impl VkInstance {
  pub fn new(window: &VkWindow) -> VkInstance {
    let entry: Entry = unsafe { Entry::new().unwrap() };
    let instance: Instance = create_instance(&entry, window);
    
    let (debug_utils_loader, debug_call_back) = create_debug_utils(&entry, &instance);
    
    VkInstance {
      entry,
      instance,
      debug_utils_loader,
      debug_call_back,
    }
  }
  
  pub fn entry(&self) -> &Entry {
    &self.entry
  }
  
  pub fn internal(&self) -> &Instance {
    &self.instance
  }
}

impl InstanceV1_0 for VkInstance {
  type Device = ash::Device;
  
  fn handle(&self) -> ash::vk::Instance {
    self.instance.handle()
  }
  
  fn fp_v1_0(&self) -> &vk::InstanceFnV1_0 {
    self.instance.fp_v1_0()
  }
  
  unsafe fn create_device(&self, a: vk::PhysicalDevice, b: &vk::DeviceCreateInfo, c: Option<&vk::AllocationCallbacks>) -> std::result::Result<<Self as InstanceV1_0>::Device, ash::vk::Result> { 
    self.instance.create_device(a, b, c)
  }
}

fn create_instance(entry: &Entry, window: &VkWindow) -> Instance {
  let app_name = CString::new("Maat_Graphics").unwrap();
 /* 
  let layer_names = [];//CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
  let layers_names_raw: Vec<*const i8> = layer_names
      .iter()
      .map(|raw_name| raw_name.as_ptr())
      .collect();
 */ 
  let surface_extensions = ash_window::enumerate_required_extensions(window.internal()).unwrap();
  let mut extension_names_raw = surface_extensions
    .iter()
    .map(|ext| ext.as_ptr())
    .collect::<Vec<_>>();
  extension_names_raw.push(DebugUtils::name().as_ptr());

  let appinfo = vk::ApplicationInfo::builder()
    .application_name(&app_name)
    .application_version(0)
    .engine_name(&app_name)
    .engine_version(0)
    .api_version(vk::make_version(1, 1, 0));

  let create_info = vk::InstanceCreateInfo::builder()
    .application_info(&appinfo)
    .enabled_layer_names(&[])
    .enabled_extension_names(&extension_names_raw);

  let instance: Instance = unsafe {
      entry
        .create_instance(&create_info, None)
        .expect("Instance creation error")
  };

  instance
}

fn create_debug_utils(entry: &Entry, instance: &Instance)
    -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
  let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                      .message_severity(
                          vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                              | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                              | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                      )
                      .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                      .pfn_user_callback(Some(vulkan_debug_callback));

  let debug_utils_loader = DebugUtils::new(entry, instance);
  let debug_call_back = unsafe {
    debug_utils_loader
      .create_debug_utils_messenger(&debug_info, None)
      .unwrap()
  };

  (debug_utils_loader, debug_call_back)
}
