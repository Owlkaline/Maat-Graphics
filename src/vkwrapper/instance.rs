use std::borrow::Cow;
use std::env;
use std::ffi::{CStr, CString};

use ash::extensions::ext::DebugUtils;
use ash::vk::make_api_version;
//pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Entry, Instance};

use crate::vkwrapper::VkWindow;

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
    let entry: Entry = unsafe { Entry::load().expect("Vulkan failed to laod") }; //unsafe { Entry::new() };
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

fn create_instance(entry: &Entry, window: &VkWindow) -> Instance {
  let app_name = CString::new("Maat_Graphics").unwrap();

  let validation_layers_enabled = match env::var("ValLayers") {
    Ok(e) if e == "1" => true,
    _ => false,
  };

  let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
  let layers_names_raw: Vec<*const i8> = layer_names
    .iter()
    .map(|raw_name| raw_name.as_ptr())
    .collect();

  let mut extension_names = ash_window::enumerate_required_extensions(window.internal())
    .unwrap()
    .to_vec();
  extension_names.push(DebugUtils::name().as_ptr());

  let appinfo = vk::ApplicationInfo::builder()
    .application_name(&app_name)
    .application_version(0)
    .engine_name(&app_name)
    .engine_version(0)
    .api_version(make_api_version(0, 1, 0, 0));

  let create_info = vk::InstanceCreateInfo::builder()
    .application_info(&appinfo)
    .enabled_layer_names(if validation_layers_enabled {
      &layers_names_raw
    } else {
      &[]
    })
    .enabled_extension_names(&extension_names);

  let instance: Instance = unsafe {
    entry
      .create_instance(&create_info, None)
      .expect("Instance creation error")
  };

  instance
}

fn create_debug_utils(
  entry: &Entry,
  instance: &Instance,
) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
  let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
    .message_severity(
      vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
    )
    .message_type(
      vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
    )
    .pfn_user_callback(Some(vulkan_debug_callback));

  let debug_utils_loader = DebugUtils::new(entry, instance);
  let debug_call_back = unsafe {
    debug_utils_loader
      .create_debug_utils_messenger(&debug_info, None)
      .unwrap()
  };

  (debug_utils_loader, debug_call_back)
}
