use vk;

use crate::modules::Device;

use std::mem;
use std::ptr;

pub struct Shader {
  module: vk::ShaderModule,
}

impl Shader {
  pub fn new(device: &Device, shader_code: &[u8]) -> Shader {
    let mut shader_module: vk::ShaderModule = unsafe { mem::uninitialized() };
    
    let shader_code_size = mem::size_of::<u8>() * shader_code.len();
    
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
      sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      codeSize: shader_code_size,
      pCode: shader_code.as_ptr() as *const _,
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
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
  
  pub fn destroy(&mut self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    println!("Destroying Shader");
    unsafe {
      vk.DestroyShaderModule(*device, self.module, ptr::null());
    }
  }
}
