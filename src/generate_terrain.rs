use crate::gltf_interpreter::{Topology, VertexArray, NormalArray, ColourArray, IndexArray, 
                              TangentArray, FinalModel, TexCoordArray, ModelDetails, Material,
                              AlphaMode};

use crate::cgmath::{Vector3, Vector4};
use crate::math;

use image::DynamicImage;
use image::GenericImageView;

const SIZE: f32 = 800.0;
const VERTEX_COUNT: i32 = 128;

const MAX_HEIGHT: f32 = 200.0;
const MAX_PIXEL_COLOUR: i32 = 256; //* 256 * 256;

fn get_height(x: u32, z: u32, image: &image::ImageBuffer<image::Luma<u8>, std::vec::Vec<u8>>) -> f32 {
  let (width, height) = image.dimensions();
  if x < 0 || x >= width || z < 0 || z > height {
    return 0.0;
  }
  
  let mut height = image.get_pixel(x, z)[0] as f32;
  
  height /= 255.0;
  
  height -= 0.5;
  height *= MAX_HEIGHT;
  
  height
}


fn calculate_normal(x: u32, z: u32, image: &image::ImageBuffer<image::Luma<u8>, std::vec::Vec<u8>>) -> [f32; 3] {
  let height_l = get_height(x-1, z, image);
  let height_r = get_height(x+1, z, image);
  let height_d = get_height(x, z-1, image);
  let height_u = get_height(x, z+1, image);
  
  let normal = Vector3::new(height_l-height_r, 2.0, height_d - height_u);
  
  let n = math::normalise_vector3(normal);
  
  [n.x, n.y, n.z]
}

pub fn generate_terrain_from_image(image: String) -> ModelDetails {
  let image = image::open(&image.clone()).expect(&("No file or Directory at: ".to_string() + &image)).to_luma();
  
  let (width, height) = image.dimensions();
  //println!("image: {:?}", image.into_raw().len());
  //let image_data = image.clone().into_raw().clone();
  
  let vertex_count = height;
  
  let mut heights: Vec<Vec<f32>> = Vec::new();
  
  let count = VERTEX_COUNT * VERTEX_COUNT;
  
  let mut verticies: Vec<[f32; 3]> = Vec::new();
  let mut normals: Vec<[f32; 3]> = Vec::new();
  let mut uvs: Vec<[f32; 2]> = Vec::new();
  
  let mut indicies = [0; (6 * (VERTEX_COUNT-1)*(VERTEX_COUNT-1)) as usize];
  
  let mut vertex_pointer = 0;
  
  for i in 0..VERTEX_COUNT as usize {
    heights.push(Vec::new());
    for j in 0..VERTEX_COUNT as usize {
      let height = get_height(j as u32, i as u32, &image);
      
      heights[i].push(height);
      
      verticies.push([
                       (j as f32 / VERTEX_COUNT as f32 - 1.0) * SIZE,
                       height,
                       (i as f32 / VERTEX_COUNT as f32 - 1.0) * SIZE
                     ]
                    );
      
      normals.push(calculate_normal(j as u32, i as u32, &image));
      
      uvs.push([
                 j as f32 / (VERTEX_COUNT as f32 - 1.0), 
                 i as f32 / (VERTEX_COUNT as f32 - 1.0)
                ]);
                
      vertex_pointer += 1;
    }
  }
  
  let mut pointer = 0;
  for gz in 0..VERTEX_COUNT-1 {
    for gx in 0..VERTEX_COUNT-1 {
      let top_left = gz*VERTEX_COUNT+gx;
      let top_right = top_left + 1;
      let bottom_left = (gz+1)*VERTEX_COUNT+gx;
      let bottom_right = bottom_left + 1;
      indicies[pointer] = top_left as u32;
      pointer+=1;
      indicies[pointer] = bottom_left as u32;
      pointer+=1;
      indicies[pointer] = top_right as u32;
      pointer+=1;
      indicies[pointer] = top_right as u32;
      pointer+=1;
      indicies[pointer] = bottom_left as u32;
      pointer+=1;
      indicies[pointer] = bottom_right as u32;
      pointer+=1;
    }
  }
  
  let material = Material {
    name: "plain".to_string(),
    textures: Vec::new(),
  
    base_colour_factor: Vector4::new(0.0, 0.6, 0.0, 1.0),
    base_colour_texture: None,
    metallic_factor: 0.0,
    roughness_factor: 1.0,
    metallic_roughness_texture: None,
    normal_texture_scale: 1.0,
    normal_texture: None,
    occlusion_texture: None,
    occlusion_texture_strength: 0.0,
    emissive_texture: None,
    emissive_factor: Vector3::new(0.0, 0.0, 0.0),
    alpha_mode: AlphaMode::Blend,
    alpha_cutoff: 0.0,
    double_sided: false,
  };
  
  let f_model = FinalModel {
    vertices: VertexArray { morph_index: 0, vertex: verticies },
    indices: IndexArray { index: indicies.to_vec() },
    normals: NormalArray { normal: normals },
    tangents: TangentArray { tangent: Vec::new() },
    texcoords: TexCoordArray { texcoord: uvs },
    colours: ColourArray { colour: Vec::new() },
    material,
    topology: Topology::TriangleList,
    has_indices: true,
    has_normals: true,
    has_tangents: false,
  };
  
  ModelDetails {
    models: vec!(f_model),
    size: Vector3::new(SIZE, 0.0, SIZE),
    height_points: Some(heights),
  }
}

pub fn generate_flat_terrain() -> ModelDetails {
  let count = VERTEX_COUNT * VERTEX_COUNT;
  
  let mut verticies: Vec<[f32; 3]> = Vec::new();
  let mut normals: Vec<[f32; 3]> = Vec::new();
  let mut uvs: Vec<[f32; 2]> = Vec::new();
  
  let mut indicies = [0; (6 * (VERTEX_COUNT-1)*(VERTEX_COUNT-1)) as usize];
  
  let mut vertex_pointer = 0;
  
  for i in 0..VERTEX_COUNT {
    for j in 0..VERTEX_COUNT {
      verticies.push([
                       (j as f32 / VERTEX_COUNT as f32 - 1.0) * SIZE,
                       0.0,
                       (i as f32 / VERTEX_COUNT as f32 - 1.0) * SIZE
                     ]
                    );
      
      normals.push([0.0, 1.0, 0.0]);
      
      uvs.push([
                 j as f32 / (VERTEX_COUNT as f32 - 1.0), 
                 i as f32 / (VERTEX_COUNT as f32 - 1.0)
                ]);
                
      vertex_pointer += 1;
    }
  }
  
  let mut pointer = 0;
  for gz in 0..VERTEX_COUNT-1 {
    for gx in 0..VERTEX_COUNT-1 {
      let top_left = gz*VERTEX_COUNT+gx;
      let top_right = top_left + 1;
      let bottom_left = (gz+1)*VERTEX_COUNT+gx;
      let bottom_right = bottom_left + 1;
      indicies[pointer] = top_left as u32;
      pointer+=1;
      indicies[pointer] = bottom_left as u32;
      pointer+=1;
      indicies[pointer] = top_right as u32;
      pointer+=1;
      indicies[pointer] = top_right as u32;
      pointer+=1;
      indicies[pointer] = bottom_left as u32;
      pointer+=1;
      indicies[pointer] = bottom_right as u32;
      pointer+=1;
    }
  }
  
  let material = Material {
    name: "plain".to_string(),
    textures: Vec::new(),
  
    base_colour_factor: Vector4::new(0.0, 0.6, 0.0, 1.0),
    base_colour_texture: None,
    metallic_factor: 0.0,
    roughness_factor: 1.0,
    metallic_roughness_texture: None,
    normal_texture_scale: 1.0,
    normal_texture: None,
    occlusion_texture: None,
    occlusion_texture_strength: 0.0,
    emissive_texture: None,
    emissive_factor: Vector3::new(0.0, 0.0, 0.0),
    alpha_mode: AlphaMode::Blend,
    alpha_cutoff: 0.0,
    double_sided: false,
  };
  
  let f_model = FinalModel {
    vertices: VertexArray { morph_index: 0, vertex: verticies },
    indices: IndexArray { index: indicies.to_vec() },
    normals: NormalArray { normal: normals },
    tangents: TangentArray { tangent: Vec::new() },
    texcoords: TexCoordArray { texcoord: uvs },
    colours: ColourArray { colour: Vec::new() },
    material,
    topology: Topology::TriangleList,
    has_indices: true,
    has_normals: true,
    has_tangents: false,
  };
  
  ModelDetails {
    models: vec!(f_model),
    size: Vector3::new(SIZE, 0.0, SIZE),
    height_points: None,
  }
}
