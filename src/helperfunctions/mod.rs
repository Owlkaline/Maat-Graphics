use graphics::Vertex3d;

pub mod vulkan_helper;
pub mod vulkan_2d;
pub mod vulkan_3d;

pub mod opengl_helper;

pub fn convert_to_vertex3d(vertex: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Vertex3d {
  Vertex3d {
    position: vertex,
    normal: normal,
    uv: uv,
  }
}
