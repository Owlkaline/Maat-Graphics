use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device, Entry, Instance};
use std::borrow::Cow;
use std::cell::RefCell;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::ops::Drop;

use crate::modules::{VkDevice};


