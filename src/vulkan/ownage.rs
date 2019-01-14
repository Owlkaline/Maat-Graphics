use vk;

use std::sync::Arc; 
use std::ops::Deref; 

pub unsafe trait SafeDeref: Deref {}
unsafe impl<'a, T: ?Sized> SafeDeref for &'a T {
}
unsafe impl<T: ?Sized> SafeDeref for Arc<T> {
}
unsafe impl<T: ?Sized> SafeDeref for Box<T> {
}

pub enum OwnedOrRef<T: 'static> {
    _Owned(T),
    Ref(&'static T),
}

impl<T> Deref for OwnedOrRef<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match *self {
            OwnedOrRef::_Owned(ref v) => v,
            OwnedOrRef::Ref(v) => v,
        }
    }
}

pub fn check_errors(result: vk::Result) -> bool {
    match result {
        vk::SUCCESS => true,
        vk::NOT_READY => { println!("Success: A fence or query has not yet completed"); true },
        vk::TIMEOUT => { println!("Success: A wait operation has not completed in the specified time"); true },
        vk::EVENT_SET => { println!("Success: An event is signaled"); true },
        vk::EVENT_RESET => { println!("Success: An event is unsignaled"); true },
        vk::INCOMPLETE => {println!("Success: A return array was too small for the result"); true },
        vk::ERROR_OUT_OF_HOST_MEMORY => panic!("Vulkan out of host memory"),
        vk::ERROR_OUT_OF_DEVICE_MEMORY => panic!("Vulkan out of device memory"),
        vk::ERROR_INITIALIZATION_FAILED => panic!("Vulkan initialization failed"),
        vk::ERROR_DEVICE_LOST => panic!("Vulkan device lost"),
        vk::ERROR_MEMORY_MAP_FAILED => panic!("Vulkan memorymap failed"),
        vk::ERROR_LAYER_NOT_PRESENT => panic!("Vulkan layer not present"),
        vk::ERROR_EXTENSION_NOT_PRESENT => panic!("Vulkan extension not present"),
        vk::ERROR_FEATURE_NOT_PRESENT => panic!("Vulkan feature not present"),
        vk::ERROR_INCOMPATIBLE_DRIVER => panic!("Vulkan incompatable driver"),
        vk::ERROR_TOO_MANY_OBJECTS => panic!("Vulkan too many objects"),
        vk::ERROR_FORMAT_NOT_SUPPORTED => panic!("Vulkan format not supported"),
        vk::ERROR_SURFACE_LOST_KHR => panic!("Vulkan surface last khr"),
        vk::ERROR_NATIVE_WINDOW_IN_USE_KHR => panic!("Vulkan window in use khr"),
        vk::SUBOPTIMAL_KHR => panic!("Vulkan suboptimal khr"),
        vk::ERROR_OUT_OF_DATE_KHR => panic!("Vulkan out of date khr"),
        vk::ERROR_INCOMPATIBLE_DISPLAY_KHR => panic!("Vulkan incompatable display khr"),
        vk::ERROR_VALIDATION_FAILED_EXT => panic!("Vulkan validation failed ext"),
        vk::ERROR_OUT_OF_POOL_MEMORY_KHR => panic!("Vulkan of out pool memory khr"),
        vk::ERROR_INVALID_SHADER_NV => panic!("Vulkan function returned \
                                               VK_ERROR_INVALID_SHADER_NV"),
        c => unreachable!("Unexpected error code returned by Vulkan: {}", c),
    }
}
