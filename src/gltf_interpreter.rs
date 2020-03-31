use std::{fs, io};
use std::path::Path;

use crate::vulkan::vkenums::{AddressMode, Filter};
use crate::math;

use base64;

use gltf;
use gltf::json::Value;
use gltf::animation;
pub use gltf::material::AlphaMode;
use gltf::texture::MagFilter;
use gltf::texture::MinFilter;
use gltf::texture::WrappingMode;

//use gltf_importer;
//use gltf_importer::config::ValidationStrategy;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::Quaternion;

use image;
use image::ImageFormat::{JPEG, PNG};

use std::mem;

pub enum Interpolation {
  Linear,
  Step,
  CatmullRomSpline,
  CubicSpline,
}

#[derive(Clone)]
pub enum Property {
  Translation,
  Rotation,
  Scale,
  MorphTargetWeights,
}

pub struct Animation {
  _interpolation: Interpolation,
  _property: Property,
  _inputs: Vector2<f32>,
  _outputs: Vector2<Vector4<f32>>,
}

impl Animation {
  pub fn new(_interpolation: Interpolation, _property: Property, _inputs: Vector2<f32>, _outputs: Vector2<Vector4<f32>>) -> Animation {
    Animation {
      _interpolation,
      _property,
      _inputs,
      _outputs,
    }
  }
}

impl Interpolation {
  pub fn from_gltf(interpolation: animation::Interpolation) -> Interpolation {
    match interpolation {
      animation::Interpolation::Linear => {
        Interpolation::Linear
      },
      animation::Interpolation::Step => {
        Interpolation::Step
      },
     /* animation::Interpolation::CatmullRomSpline => {
        Interpolation::CatmullRomSpline
      },*/
      animation::Interpolation::CubicSpline => {
        Interpolation::CubicSpline
      }
    }
  }
}

impl Property {
  pub fn from_gltf(property: animation::Property) -> Property {
    match property {
      animation::Property::Translation => {
        Property::Translation
      },
      animation::Property::Rotation => {
        Property::Rotation
      },
      animation::Property::Scale => {
        Property::Scale
      },
      animation::Property::MorphTargetWeights => {
        Property::MorphTargetWeights
      }
    }
  }
}

#[derive(Clone)]
pub enum Topology {
  PointList,
  LineList,
  _LineLoop, // Vulkan not supoprted
  LineStrip,
  TriangleList,
  TriangleStrip,
  TriangleFan,
}

#[derive(Clone)]
pub struct IndexArray {
  pub index: Vec<u32>,
}

#[derive(Clone)]
pub struct TexCoordArray {
  pub texcoord: Vec<[f32; 2]>
}

#[derive(Clone)]
pub struct TangentArray {
  pub tangent: Vec<[f32; 4]>,
}

#[derive(Clone)]
pub struct NormalArray {
  pub normal: Vec<[f32; 3]>,
}

#[derive(Clone)]
pub struct VertexArray {
  pub morph_index: u32,
  pub vertex: Vec<[f32; 3]>,
}

#[derive(Clone)]
pub struct ColourArray {
  pub colour: Vec<[f32; 4]>,
}

#[derive(Clone)]
pub struct SamplerInfo {
  pub mag_filter: Filter,
  pub min_filter: Filter,
  pub s_wrap: AddressMode,
  pub t_wrap: AddressMode,
}

#[derive(Clone)]
pub struct Texture {
  texture: String,
  
  raw_transform: [f32; 16],
}

#[derive(Clone)]
pub struct Material {
  pub name: String,
  pub textures: Vec<Texture>,
  
  pub base_colour_factor: Vector4<f32>,
  pub base_colour_texture: Option<(Option<image::DynamicImage>, SamplerInfo)>,
  pub metallic_factor: f32,
  pub roughness_factor: f32,
  pub metallic_roughness_texture: Option<(Option<image::DynamicImage>, SamplerInfo)>,
  pub normal_texture_scale: f32,
  pub normal_texture: Option<(Option<image::DynamicImage>, SamplerInfo)>,
  pub occlusion_texture: Option<(Option<image::DynamicImage>, SamplerInfo)>,
  pub occlusion_texture_strength: f32,
  pub emissive_texture: Option<(Option<image::DynamicImage>, SamplerInfo)>,
  pub emissive_factor: Vector3<f32>,
  pub alpha_mode: AlphaMode,
  pub alpha_cutoff: f32,
  pub double_sided: bool,
}

#[derive(Clone)]
pub struct FinalModel {
  pub vertices: VertexArray,
  pub indices: IndexArray,
  pub normals: NormalArray,
  pub tangents: TangentArray,
  pub texcoords: TexCoordArray,
  pub colours: ColourArray,
  pub material: Material,
  pub topology: Topology,
  pub has_indices: bool,
  pub has_normals: bool,
  pub has_tangents: bool,
//  animation: Animation,
}

#[derive(Clone)]
pub struct ModelDetails {
  pub models: Vec<FinalModel>,
  pub size: Vector3<f32>,
 // materials: Vec<Material>,
}

impl Drop for ModelDetails{
  fn drop(&mut self) {
    let _children = mem::replace(&mut self.models, Vec::new());
    /*
    loop {
      children = match children {
        Some(mut n) => mem::replace(&mut n.borrow_mut().models, Vec::new()),
        None => break,
      }
    }*/
  }
}

impl Texture {
  pub fn _new() -> Texture {
    Texture {
      texture: "".to_string(),
      raw_transform: [1.0, 0.0, 0.0, 0.0, 
                      0.0, 1.0, 0.0, 0.0, 
                      0.0, 0.0, 1.0, 0.0, 
                      0.0, 0.0, 0.0, 1.0],
    }
  }
}

impl Material {
 pub fn new() -> Material {
   Material {
     name: "".to_string(),
     textures: Vec::new(),
     
     base_colour_factor: Vector4::new(0.0, 0.0, 0.0, 1.0),
     base_colour_texture: None,
     metallic_factor: 0.0,
     roughness_factor: 0.0,
     metallic_roughness_texture: None,
     normal_texture_scale: 0.0,
     normal_texture: None,
     occlusion_texture: None,
     occlusion_texture_strength: 0.0,
     emissive_texture: None,
     emissive_factor: Vector3::new(0.0, 0.0, 0.0),
     alpha_mode: AlphaMode::Blend,
     alpha_cutoff: 0.5,
     double_sided: false,
   }
 }
}
/*
impl Sampler {
  pub fn new() -> Sampler {
    Sampler {
      mag_filter: MagFilter::Linear,
      min_filter: MinFilter::Linear,
      wrap_s: WrappingMode::ClampToEdge,
      wrap_t: WrappingMode::ClampToEdge,
    }
  }
}*/

fn serde_to_f32(value: Option<Value>) -> Vector4<f32> {
  let floats = value.and_then(|value| { 
    value.as_array().map(|v_array| {
      v_array.iter().map(|v| {
        v.as_f64().map(|f| {
         f as f32 
        })
      }).flatten().collect::<Vec<f32>>()
    })
  }).unwrap_or(Vec::new());
  
  let mut serde_values = Vector4::new(0.0, 0.0, 0.0, 0.0);
  
  for i in 0..floats.len().min(4) {
    serde_values[i] = floats[i];
  }
  
  serde_values
}

impl ModelDetails {
  pub fn new(source: String) -> ModelDetails {
    let source = &source;
    
    let mut points: Vec<Vec<f32>> = Vec::new();
    //let (gltf, buffers, images) = gltf::import("./examples/ObjectStatic.gltf").unwrap();
//    let source = "./examples/ObjectStatic.gltf";
    /*
    let config = gltf_importer::Config { validation_strategy: ValidationStrategy::Complete };
    let (gltf, buffers) = match gltf_importer::import_with_config(source, &config) {
      Ok((gltf, buffers)) => (gltf, buffers),
      Err(err) => {
        panic!("glTF import failed: {:?}", err);
        if let gltf_importer::Error::Io(_) = err {
          panic!("Hint: Are the .bin file(s) referenced by the .gltf file available?")
        }
        (gltf::gltf::Gltf, gltf_importer::Buffers)
      },
    };*/
    //println!("{}", source);
    let load = gltf::import(source);
    let (gltf, buffers, _images) = match load {
      Ok(t) => {
        t
      },
      Err(e) => {
        println!("{:?}", e);
        panic!(e);
      }
    };
//    let (gltf, buffers, images) = .unwrap();
    
    /*
    println!("{:?}", buffers);
    for scene in gltf.scenes() {
      print!("Scene {}", scene.index());
      #[cfg(feature = "names")]
      print!(" ({})", scene.name().unwrap_or("<Unnamed>"));
      println!();
      for node in scene.nodes() {
        print_tree(&node, 1);
      }
    }*/
    

    let mut models: Vec<FinalModel> = Vec::new();
    let mut min_xyz: Vector3<f32> = Vector3::new(10000000000000000.0, 100000000000000.0, 10000000000000000.0);
    let mut max_xyz: Vector3<f32> = Vector3::new(-10000000000000000000.0, -100000000000000000.0, -100000000000000000.0);
  /*  let mut models = FinalModel {
      vertices: VertexArray,
      indices: IndexArray,
      normals: NormalArray,
      texcoords: TexCoordArray,
      material_ref: String,
      animation: Animation,
  }*/
    
    let mut animations: Vec<Animation> = Vec::new();
    
    for animation in gltf.animations() {
      print!("Animation Name: ");
      if let Some(anim_name) = animation.name() {
        println!("{}", anim_name.to_string());
      } else {
        println!("NoName");
      }
      
      let mut properties: Vec<Property> = Vec::new(); 
      
      for channel in animation.channels() {
        println!("Property {:?}", channel.target().property());
        let property = Property::from_gltf(channel.target().property());
        
        properties.push(property);
      }
      
      let mut i = 0;
      for sampler in animation.samplers() {
        let input_min: f32 = serde_to_f32(sampler.input().min()).x;
        let input_max: f32 = serde_to_f32(sampler.input().max()).x;
        let output_min: Vector4<f32> = serde_to_f32(sampler.output().min());
        let output_max: Vector4<f32> = serde_to_f32(sampler.output().max());
        
        let interpolation = Interpolation::from_gltf(sampler.interpolation());
        
        println!("Interpolation: {:?}", sampler.interpolation());
        println!("in min: {:?}, expected: {:?}", input_min, sampler.input().min());
        println!("out min: {:?}, expected: {:?}", output_min, sampler.output().min());
        println!("");
        println!("in max: {:?}, expected: {:?}", input_max, sampler.input().max());
        println!("out max: {:?}, expected: {:?}", output_max, sampler.output().max());
        println!("");
        println!("");
        
        let inputs: Vector2<f32> = Vector2::new(input_min, input_max);
        let outputs: Vector2<Vector4<f32>> = Vector2::new(output_min, output_max);
        
        animations.push(Animation::new(interpolation, properties[i].clone(), inputs, outputs));
        
        i += 1;
      }
    }
    
    let gltf_textures_samplers: Vec<(Option<image::DynamicImage>, SamplerInfo)> = {
      let mut textures_samplers = Vec::new();
      
      for texture in gltf.textures() {
        //println!("Texture: {:?}", texture.source().index());
        let img: Option<image::DynamicImage> = texture_to_image(texture.clone(), &buffers, &Path::new(&source));
        
        let mag_filter = {
          match texture.sampler().mag_filter().unwrap_or(MagFilter::Linear) {
            MagFilter::Linear => {
              Filter::Linear
            },
            MagFilter::Nearest => {
              Filter::Nearest
            }
          }
        };
        
        let min_filter = {
          match texture.sampler().min_filter().unwrap_or(MinFilter::Linear) {
            MinFilter::Linear => {
              Filter::Linear
            },
            MinFilter::Nearest => {
              Filter::Nearest
            },
            _ => {Filter::Linear},
          }
        };
        
        let s_wrap = {
          match texture.sampler().wrap_s() {
            WrappingMode::ClampToEdge => {
              AddressMode::ClampToEdge
            },
            WrappingMode::MirroredRepeat => {
              AddressMode::MirroredRepeat
            },
            WrappingMode::Repeat => {
              AddressMode::Repeat
            }
          }
        };
        
        let t_wrap = {
          match texture.sampler().wrap_t() {
            WrappingMode::ClampToEdge => {
              AddressMode::ClampToEdge
            },
            WrappingMode::MirroredRepeat => {
              AddressMode::MirroredRepeat
            },
            WrappingMode::Repeat => {
              AddressMode::Repeat
            }
          }
        };
        
        let samplerinfo = SamplerInfo {
                        mag_filter,
                        min_filter,
                        s_wrap,
                        t_wrap,
                      };
        
        textures_samplers.push((img, samplerinfo));
      }
      
      textures_samplers
    };
    
    let mut index = 0;
    for node in gltf.nodes() {
      let transform_matrix = node.transform().matrix();
      let t_matrix = Matrix4::from_cols(
                                Vector4::new(transform_matrix[0][0], transform_matrix[0][1], transform_matrix[0][2], transform_matrix[0][3]),
                                Vector4::new(transform_matrix[1][0], transform_matrix[1][1], transform_matrix[1][2], transform_matrix[1][3]),
                                Vector4::new(transform_matrix[2][0], transform_matrix[2][1], transform_matrix[2][2], transform_matrix[2][3]),
                                Vector4::new(transform_matrix[3][0], transform_matrix[3][1], transform_matrix[3][2], transform_matrix[3][3])
                              );
      //let (translation, quaternion, scale) = node.transform().decomposed();
      //let scale = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
      //let translation = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
      //println!("{:?}", quaternion);
      //let quaternion = math::array4_to_vec4(quaternion);
      //let quaternion = math::quaternion_to_axis_angle(quaternion);
      //println!("{:?}\n", quaternion);
      for mesh in node.mesh() {
        //println!("{}", index);
        //println!("Mesh #{}", mesh.index());
        
        for primitive in mesh.primitives() {
          models.push(FinalModel {
            vertices: VertexArray { vertex: Vec::new(), morph_index: 0 }, 
            indices: IndexArray { index: Vec::new() }, 
            normals: NormalArray { normal: Vec::new() },
            tangents: TangentArray { tangent: Vec::new() },
            texcoords: TexCoordArray { texcoord: Vec::new() },
            colours: ColourArray { colour: Vec::new() },
            material: Material::new(),
            topology: Topology::TriangleStrip, // default
            //  animation: Animation::new(),
            has_indices: false,
            has_normals: false,
            has_tangents: false,
          });
          
          let bounding_box = primitive.bounding_box();
          
          min_xyz.x = min_xyz.x.min(bounding_box.min[0]);
          min_xyz.y = min_xyz.y.min(bounding_box.min[1]);
          min_xyz.z = min_xyz.z.min(bounding_box.min[2]);
          
          max_xyz.x = max_xyz.x.max(bounding_box.max[0]);
          max_xyz.y = max_xyz.y.max(bounding_box.max[1]);
          max_xyz.z = max_xyz.z.max(bounding_box.max[2]);
          
          //println!("- Primitive #{}", primitive.index());
          //println!("Material: {:?}", primitive.material().index());
          //println!("Material name: {:?}", primitive.material().name());
          //println!("Base Colour: {:?}", primitive.material().pbr_metallic_roughness().base_color_factor());
          let mut texture_index = 0;
          if let Some(info) = primitive.material().pbr_metallic_roughness().base_color_texture() {
            texture_index = info.tex_coord();
            //println!("Base Texture name: {:?}", info.texture().source().name());
          }
          
          models[index].topology = match primitive.mode() {
            gltf::mesh::Mode::Points => Topology::PointList,
            gltf::mesh::Mode::Lines => Topology::LineList,
            gltf::mesh::Mode::LineLoop => panic!("LineLoop not supported"),
            gltf::mesh::Mode::LineStrip => Topology::LineStrip,
            gltf::mesh::Mode::Triangles => Topology::TriangleList,
            gltf::mesh::Mode::TriangleStrip => Topology::TriangleStrip,
            gltf::mesh::Mode::TriangleFan => Topology::TriangleFan,
          };
          
          let mat = primitive.material();
          let pbr = mat.pbr_metallic_roughness();
          
          models[index].material.alpha_mode = mat.alpha_mode();
          models[index].material.alpha_cutoff = mat.alpha_cutoff();
          models[index].material.double_sided = mat.double_sided();
          
          let colour_factor = pbr.base_color_factor();
          models[index].material.base_colour_factor = Vector4::new(colour_factor[0], colour_factor[1], colour_factor[2], colour_factor[3]);
//          models[mesh.index()].material.base_color_texture_tex_coord = pbr.base_color_texture().map(|t| t.tex_coord() as i32).unwrap_or(-1);
          models[index].material.base_colour_texture = pbr.base_color_texture().map(|t| {
            gltf_textures_samplers[t.texture().index()].clone()
          });
          models[index].material.metallic_factor = pbr.metallic_factor();
          models[index].material.roughness_factor = pbr.roughness_factor();
          models[index].material.metallic_roughness_texture = pbr.metallic_roughness_texture().map(|t| gltf_textures_samplers[t.texture().index()].clone());
          models[index].material.normal_texture_scale = mat.normal_texture().map(|t| t.scale()).unwrap_or(0.0);
          models[index].material.normal_texture = mat.normal_texture().map(|t| gltf_textures_samplers[t.texture().index()].clone());
          models[index].material.occlusion_texture = mat.occlusion_texture().map(|t| gltf_textures_samplers[t.texture().index()].clone());
          models[index].material.occlusion_texture_strength = mat.occlusion_texture().map(|t| t.strength()).unwrap_or(0.0);
          models[index].material.emissive_texture = mat.emissive_texture().map(|t| gltf_textures_samplers[t.texture().index()].clone());
          let emissive_factor = mat.emissive_factor();
          models[index].material.emissive_factor = Vector3::new(emissive_factor[0], emissive_factor[1], emissive_factor[2]);
          
          let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
          if let Some(iter) = reader.read_positions() {
            let mut vertices = Vec::with_capacity(iter.len());
            for vertex_position in iter {
              let vertex = Vector4::new(vertex_position[0], vertex_position[1], vertex_position[2], 1.0);
              ///let vertex = math::rotate_vertex_by_angle(vertex, quaternion);
              //let vertex = (translation*scale)*Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);
              let vertex = t_matrix * vertex;
              
              vertices.push([vertex.x, vertex.y, vertex.z]);
            }
            models[index].vertices.vertex = vertices;
          }
          if let Some(iter) = reader.read_normals() {
            let mut normals = Vec::with_capacity(iter.len());
            for vertex_normal in iter {
              let normal = Vector4::new(vertex_normal[0], vertex_normal[1], vertex_normal[2], 1.0);
              //let normal = math::rotate_vertex_by_angle(normal, quaternion);
              let normal = t_matrix * normal;
              
              normals.push([normal.x, normal.y, normal.z]);
              
              models[index].has_normals = true;
            }
            models[index].normals.normal = normals;
          }
          if let Some(iter) = reader.read_tangents() {
            let mut tangents = Vec::with_capacity(iter.len());
            for vertex_tangent in iter {
              let tangent = Vector4::new(vertex_tangent[0], vertex_tangent[1], vertex_tangent[2], vertex_tangent[3]);
             // let tangent = math::rotate_vertex_by_angle(tangent.xyz(), quaternion);
              let tangent = t_matrix * tangent;
              tangents.push([tangent.x, tangent.y, tangent.z, vertex_tangent[3]]);
              
              models[index].has_tangents = true;
            }
            models[index].tangents.tangent = tangents;
          }
          if let Some(iter) = reader.read_indices() {
            let mut indices = Vec::new();
            for vertex_indices in iter.into_u32() {
              indices.push(vertex_indices);
              
              models[index].has_indices = true;
            }
            models[index].indices.index = indices;
          }
          if let Some(iter) = reader.read_tex_coords(texture_index) {
            let mut texcoords = Vec::new();
            for vertex_texcoords in iter.into_f32() {
              texcoords.push(vertex_texcoords);
            }
            models[index].texcoords.texcoord = texcoords;
          }
          if let Some(iter) = reader.read_colors(texture_index) {
            let mut colours = Vec::new();
            for vertex_colour in iter.into_rgba_f32() {
              colours.push(vertex_colour);
            }
            models[index].colours.colour = colours;
          }
          index += 1;
        }
      }
    }
    //let mut materials: Vec<Material> = Vec::new();
    
    ModelDetails {
      models: models,
      size: Vector3::new(max_xyz.x - min_xyz.x, max_xyz.y - min_xyz.y, max_xyz.z - min_xyz.z),
     // materials: materials,
    }
  }
  
  pub fn get_size(&self) -> Vector3<f32> {
    self.size
  }
  
  pub fn num_models(&self) -> usize {
    self.models.len()
  }
  
  pub fn alphamode(&self, model_index: usize) -> AlphaMode {
    self.models[model_index].material.alpha_mode
  }
  
  pub fn alphacutoff(&self, model_index: usize) -> f32 {
    self.models[model_index].material.alpha_cutoff
  }
  
  pub fn double_sided(&self, model_index: usize) -> bool {
    self.models[model_index].material.double_sided
  }
  
  pub fn _has_indices(&self, model_index: usize) -> bool {
    self.models[model_index].has_indices
  }
  
  pub fn _has_normals(&self, model_index: usize) -> bool {
    self.models[model_index].has_normals
  }
  
  pub fn _has_tangents(&self, model_index: usize) -> bool {
    self.models[model_index].has_tangents
  }
  
  pub fn vertex(&self, model_index: usize) -> Vec<[f32; 3]> {
    self.models[model_index].vertices.vertex.clone()
  }
  
  pub fn normal(&self, model_index: usize) -> Vec<[f32; 3]> {
    self.models[model_index].normals.normal.clone()
  }
  
  pub fn tangent(&self, model_index: usize) -> Vec<[f32; 4]> {
    self.models[model_index].tangents.tangent.clone()
  }
  
  pub fn index(&self, model_index: usize) -> Vec<u32> {
    self.models[model_index].indices.index.clone()
  }
  
  pub fn texcoords(&self, model_index: usize) -> Vec<[f32; 2]> {
    self.models[model_index].texcoords.texcoord.clone()
  }
  
  pub fn colours(&self, model_index: usize) -> Vec<[f32; 4]> {
    self.models[model_index].colours.colour.clone()
  }
  
  pub fn base_colour(&self, model_index: usize) -> [f32; 4] {
    let colour = self.models[model_index].material.base_colour_factor;
    [colour.x, colour.y, colour.z, colour.w]
  }
  
  pub fn base_colour_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    let mut texture = None;
    if self.models[model_index].material.base_colour_texture.is_some() {
      texture = self.models[model_index].material.base_colour_texture.clone().unwrap().0;
    }
    texture
  }
  
  pub fn base_colour_sampler(&self, model_index: usize) -> Option<SamplerInfo> {
    let mut sampler = None;
    if self.models[model_index].material.base_colour_texture.is_some() {
      sampler = Some(self.models[model_index].material.base_colour_texture.clone().unwrap().1);
    }
    sampler
  }
  
  pub fn metallic_factor(&self, model_index: usize) -> f32 {
    self.models[model_index].material.metallic_factor
  }
  
  pub fn roughness_factor(&self, model_index: usize) -> f32 {
    self.models[model_index].material.roughness_factor
  }
  
  pub fn metallic_roughness_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    let mut texture = None;
    if self.models[model_index].material.metallic_roughness_texture.is_some() {
      texture = self.models[model_index].material.metallic_roughness_texture.clone().unwrap().0;
    }
    texture
  }
  
  pub fn _metallic_roughness_sampler(&self, model_index: usize) -> Option<SamplerInfo> {
    let mut sampler = None;
    if self.models[model_index].material.metallic_roughness_texture.is_some() {
      sampler = Some(self.models[model_index].material.metallic_roughness_texture.clone().unwrap().1);
    }
    sampler
  }
  
  pub fn normal_texture_scale(&self, model_index: usize) -> f32 {
    self.models[model_index].material.normal_texture_scale
  }
  
  pub fn normal_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    let mut texture = None;
    if self.models[model_index].material.normal_texture.is_some() {
      texture = self.models[model_index].material.normal_texture.clone().unwrap().0;
    }
    texture
  }
  
  pub fn _normal_sampler(&self, model_index: usize) -> Option<SamplerInfo> {
    let mut sampler = None;
    if self.models[model_index].material.normal_texture.is_some() {
      sampler = Some(self.models[model_index].material.normal_texture.clone().unwrap().1);
    }
    sampler
  }
  
  pub fn occlusion_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    let mut texture = None;
    if self.models[model_index].material.occlusion_texture.is_some() {
      texture = self.models[model_index].material.occlusion_texture.clone().unwrap().0
    }
    texture
  }
  
  pub fn _occlusion_sampler(&self, model_index: usize) -> Option<SamplerInfo> {
    let mut sampler = None;
    if self.models[model_index].material.occlusion_texture.is_some() {
      sampler = Some(self.models[model_index].material.occlusion_texture.clone().unwrap().1);
    }
    sampler
  }
  
  pub fn occlusion_texture_strength(&self, model_index: usize) -> f32 {
    self.models[model_index].material.occlusion_texture_strength
  }
  
  pub fn emissive_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    let mut texture = None;
    if self.models[model_index].material.emissive_texture.is_some() {
      texture = self.models[model_index].material.emissive_texture.clone().unwrap().0
    }
    texture
  }
  
  pub fn _emissive_sampler(&self, model_index: usize) -> Option<SamplerInfo> {
    let mut sampler = None;
    if self.models[model_index].material.emissive_texture.is_some() {
      sampler = Some(self.models[model_index].material.emissive_texture.clone().unwrap().1);
    }
    sampler
  }
  
  pub fn emissive_factor(&self, model_index: usize) -> [f32; 3] {
    let emissive = self.models[model_index].material.emissive_factor;
    [emissive.x, emissive.y, emissive.z]
  }
  
  pub fn _get_primitive_topology(&self, model_index: usize) -> Topology {
    self.models[model_index].topology.clone()
  }
}

fn texture_to_image(texture: gltf::Texture, buffers: &Vec<gltf::buffer::Data>, base_path: &Path) -> Option<image::DynamicImage> {
  let data = texture.source().source();
  let img = match data {
    gltf::image::Source::View { ref view, mime_type } => {
      let data = &buffers[view.buffer().index()].0;
      let begin = view.offset();
      let end = begin + view.length();
      let real_data = &data[begin..end];
      //let data = &buffers[view.offset()].to_vec();
      //buffers.view(view).expect("Failed to get buffer view for image");
      match mime_type {
        "image/jpeg" => image::load_from_memory_with_format(real_data, JPEG),
        "image/png" => image::load_from_memory_with_format(real_data, PNG),
        _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                    texture.index(), mime_type)),
      }
    },
    gltf::image::Source::Uri { uri, mime_type } => {
      //println!("{:?}", uri);
      if uri.starts_with("data:") {
        let encoded = uri.split(',').nth(1).unwrap();
        let data = base64::decode(&encoded).unwrap();
        let mime_type = if let Some(ty) = mime_type {
          ty
        } else {
          uri.split(',')
          .nth(0).unwrap()
          .split(':')
          .nth(1).unwrap()
          .split(';')
          .nth(0).unwrap()
        };
        
        match mime_type {
          "image/jpeg" => image::load_from_memory_with_format(&data, JPEG),
          "image/png" => image::load_from_memory_with_format(&data, PNG),
          _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                      texture.index(), mime_type)),
        }
      }
      else {
        if let Some(mime_type) = mime_type {
          let path = base_path.parent().unwrap_or_else(|| Path::new("./")).join(uri);
          let file = fs::File::open(path).unwrap();
          let reader = io::BufReader::new(file);
          match mime_type {
            "image/jpeg" => image::load(reader, JPEG),
            "image/png" => image::load(reader, PNG),
            _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                         texture.index(), mime_type)),
          }
        }
        else {
          let path = base_path.parent().unwrap_or_else(||Path::new("./")).join(uri);
          image::open(path)
        }
      }
    }
  };
  
  Some(img.unwrap())
}

/*

fn print_tree(node: &gltf::Node, depth: i32) {
    for _ in 0..(depth - 1) {
        print!("  ");
    }
    print!(" -");
    print!(" Node {}", node.index());
    #[cfg(feature = "names")]
    print!(" ({})", node.name().unwrap_or("<Unnamed>"));
    println!();

    for child in node.children() {
        print_tree(&child, depth + 1);
    }
}

  let primitives: Vec<Primitive> = g_mesh.primitives()
            .enumerate()
            .map(|(i, g_prim)| {
                Primitive::from_gltf(&g_prim, i, g_mesh.index(), root, buffers, base_path)
            })
            .collect();*/
            
