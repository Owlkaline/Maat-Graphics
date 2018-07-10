use graphics::Vertex3d;

pub fn convert_to_vertex3d(vertex: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Vertex3d {
  Vertex3d {
    position: vertex,
    normal: normal,
    uv: uv,
  }
}
