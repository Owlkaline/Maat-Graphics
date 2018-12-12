use vk;

use modules::Device;

use std::mem;
use std::ptr;

pub struct Shader {
  module: vk::ShaderModule,
}

impl Shader {
  pub fn new(device: &Device, shader_code: &[u8]) -> Shader {
    let mut shader_module: vk::ShaderModule = unsafe { mem::uninitialized() };
    
    let mut shader_code_size = mem::size_of::<u8>() * shader_code.len();
    
    let mut shader_module_create_info = vk::ShaderModuleCreateInfo {
      sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      codeSize: shader_code_size,
      pCode: shader_code.as_ptr() as *const _,
    };
    
    let vk = device.pointers();
    let device = device.local_device();
    
    unsafe {
      vk.CreateShaderModule(*device, &shader_module_create_info, ptr::null(), &mut shader_module);
    }
    
    Shader {
      module: shader_module,
    }
  }
  
  pub fn get_shader(&self) -> &vk::ShaderModule {
    &self.module
  }
  
  pub fn destroy(&mut self, vk: &vk::DevicePointers, device: &vk::Device) {
    unsafe {
      vk.DestroyShaderModule(*device, self.module, ptr::null());
    }
  }
}

impl Drop for Shader {
  fn drop(&mut self) {
    unsafe {
   //   self.vk.DestroyShaderModule(self.device, self.module, ptr::null());
    }
  }
}
