pub use self::vkmaat::VkMaat;
pub use self::final_shader::FinalShader;
pub use self::texture_shader::TextureShader;
pub use self::resource::ResourceManager;

mod vkmaat;
mod resource;
mod final_shader;
mod texture_shader;
//mod vkcreate;
//mod vulkan_2d;
//mod vulkan_3d;
//mod vulkan_draw;
//mod vulkan_helper;
//mod renderpass;

pub mod vs_texture {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkTexture.vert"]
  struct Dummy;
}

pub mod fs_texture {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkTexture.frag"]
  struct Dummy;
}

pub mod vs_text {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkText.vert"]
  struct Dummy;
}

pub mod fs_text {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkText.frag"]
  struct Dummy;
}

pub mod vs_forwardbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkForward3D.vert"]
  struct Dummy;
}

pub mod fs_forwardbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkForward3D.frag"]
  struct Dummy;
}

pub mod vs_gbuffer_3d{
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkGBuffer3D.vert"]
  struct Dummy;
}

pub mod fs_gbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkGBuffer3D.frag"]
  struct Dummy;
}

pub mod vs_plain {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPlain.vert"]
  struct Dummy;
}

pub mod fs_lights {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkLight.frag"]
  struct Dummy;
}

pub mod fs_post_bloom {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBloom.frag"]
  struct Dummy;
}

pub mod vs_shadow {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkShadow.vert"]
  struct Dummy;
}

pub mod fs_shadow {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkShadow.frag"]
  struct Dummy;
}

mod vs_post_blur {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPostBlur.vert"]
  struct Dummy;
}

mod fs_post_blur {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBlur.frag"]
  struct Dummy;
}
