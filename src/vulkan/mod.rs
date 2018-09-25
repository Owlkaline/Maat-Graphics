pub use self::vkmaat::VkMaat;
pub use self::final_shader::FinalShader;
pub use self::texture_shader::TextureShader;
pub use self::resource_manager::ResourceManager;

mod vkmaat;
mod resource_manager;
mod final_shader;
mod texture_shader;
/*
pub mod vs_forwardbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkForward3D.vert"]
  struct _Dummy;
}

pub mod fs_forwardbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkForward3D.frag"]
  struct _Dummy;
}

pub mod vs_gbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkGBuffer3D.vert"]
  struct _Dummy;
}

pub mod fs_gbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkGBuffer3D.frag"]
  struct _Dummy;
}

pub mod vs_plain {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPlain.vert"]
  struct _Dummy;
}

pub mod fs_lights {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkLight.frag"]
  struct _Dummy;
}

pub mod fs_post_bloom {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBloom.frag"]
  struct _Dummy;
}

pub mod vs_shadow {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkShadow.vert"]
  struct _Dummy;
}

pub mod fs_shadow {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkShadow.frag"]
  struct _Dummy;
}

mod vs_post_blur {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPostBlur.vert"]
  struct _Dummy;
}

mod fs_post_blur {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBlur.frag"]
  struct _Dummy;
}*/
