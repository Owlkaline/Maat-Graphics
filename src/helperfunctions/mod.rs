use graphics::Vertex3d;

pub fn convert_to_vertex3d(vertex: [f32; 3], normal: [f32; 3], tangent: [f32; 4], uv: [f32; 2], colour: [f32; 4]) -> Vertex3d {
  Vertex3d {
    position: vertex,
    normal: normal,
    tangent: tangent,
    uv: uv,
    colour: colour,
  }
}
