use std::{fs, io};
use std::path::Path;

use base64;

use gltf;
use gltf::json::texture::MinFilter;
//use gltf_importer;
//use gltf_importer::config::ValidationStrategy;

use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::Quaternion;
use cgmath::Rotation;

use image;
use image::ImageFormat::{JPEG, PNG};
use image::DynamicImage::*;
use image::GenericImage;
use image::FilterType;

#[derive(Clone)]
pub enum Topology {
  PointList,
  LineList,
  LineLoop, // Vulkan not supoprted
  LineStrip,
  TriangleList,
  TriangleStrip,
  TriangleFan,
}

#[derive(Clone)]
struct IndexArray {
  index: Vec<u32>,
}

#[derive(Clone)]
struct TexCoordArray {
  texcoord: Vec<[f32; 2]>
}

#[derive(Clone)]
struct NormalArray {
  normal: Vec<[f32; 3]>,
}

#[derive(Clone)]
struct VertexArray {
  morph_index: u32,
  vertex: Vec<[f32; 3]>,
}

#[derive(Clone)]
struct Texture {
  texture: String,
  
  raw_transform: [f32; 16],
}

#[derive(Clone)]
struct Material {
  name: String,
  textures: Vec<Texture>,
  
  base_colour_factor: Vector4<f32>,
  base_colour_texture: Option<image::DynamicImage>,
  metallic_factor: f32,
  roughness_factor: f32,
  metallic_roughness_texture: Option<image::DynamicImage>,
  normal_texture_scale: f32,
  normal_texture: Option<image::DynamicImage>,
  occlusion_texture: Option<image::DynamicImage>,
  occlusion_texture_strength: f32,
  emissive_texture: Option<image::DynamicImage>,
  emissive_factor: Vector3<f32>,
}

#[derive(Clone)]
struct FinalModel {
  vertices: VertexArray,
  indices: IndexArray,
  normals: NormalArray,
  texcoords: TexCoordArray,
  material: Material,
  topology: Topology,
//  animation: Animation,
}

pub struct ModelDetails {
  models: Vec<FinalModel>,
 // materials: Vec<Material>,
}

impl Texture {
  pub fn new() -> Texture {
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
   }
 }
}

impl ModelDetails {
  pub fn new(source: String) -> ModelDetails {
    let source = &source;
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
    println!("{}", source);
    let (gltf, buffers, images) = gltf::import(source).unwrap();
    
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
  /*  let mut models = FinalModel {
      vertices: VertexArray,
      indices: IndexArray,
      normals: NormalArray,
      texcoords: TexCoordArray,
      material_ref: String,
      animation: Animation,
  }*/
    
    let gltf_textures = {
      let mut textures = Vec::new();
      
      for texture in gltf.textures() {
        println!("Texture: {:?}", texture.source().index());
      //  let sampler = Sampler::simple_repeat_linear(queue.device().clone());
     //   for texture in gltf.textures() {
          let img: Option<image::DynamicImage> = texture_to_image(texture, &buffers, &Path::new(&source));
          textures.push(img);
      //  }
      }
      
      textures
    };
    
    let mut index = 0;
    for node in gltf.nodes() {
      let (translation, rotation, scale) = node.transform().decomposed();
      let scale = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
      let translation = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
      let quaternion = Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]);
      let inverse_quaternion = quaternion.invert();
      
      for mesh in node.mesh() {
        println!("{}", index);
        println!("Mesh #{}", mesh.index());
        
        for primitive in mesh.primitives() {
          models.push(FinalModel {
            vertices: VertexArray { vertex: Vec::new(), morph_index: 0 }, 
            indices: IndexArray { index: Vec::new() }, 
            normals: NormalArray { normal: Vec::new() }, 
            texcoords: TexCoordArray { texcoord: Vec::new() },
            material: Material::new(),
            topology: Topology::TriangleStrip, // default
            //  animation: Animation::new(),
          });
          
          println!("- Primitive #{}", primitive.index());
          println!("Material: {:?}", primitive.material().index());
          println!("Material name: {:?}", primitive.material().name());
          println!("Base Colour: {:?}", primitive.material().pbr_metallic_roughness().base_color_factor());
          let mut texture_index = 0;
          if let Some(info) = primitive.material().pbr_metallic_roughness().base_color_texture() {
            texture_index = info.tex_coord();
            println!("Base Texture name: {:?}", info.texture().source().name());
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
          
          let colour_factor = pbr.base_color_factor();
          models[index].material.base_colour_factor = Vector4::new(colour_factor[0], colour_factor[1], colour_factor[2], colour_factor[3]);
//          models[mesh.index()].material.base_color_texture_tex_coord = pbr.base_color_texture().map(|t| t.tex_coord() as i32).unwrap_or(-1);
          models[index].material.base_colour_texture = pbr.base_color_texture().map(|t| {
            gltf_textures[t.texture().index()].clone().unwrap()
          });
          models[index].material.metallic_factor = pbr.metallic_factor();
          models[index].material.roughness_factor = pbr.roughness_factor();
          models[index].material.metallic_roughness_texture = pbr.metallic_roughness_texture().map(|t| gltf_textures[t.texture().index()].clone().unwrap());
          models[index].material.normal_texture_scale = mat.normal_texture().map(|t| t.scale()).unwrap_or(0.0);
          models[index].material.normal_texture = mat.normal_texture().map(|t| gltf_textures[t.texture().index()].clone().unwrap());
          models[index].material.occlusion_texture = mat.occlusion_texture().map(|t| gltf_textures[t.texture().index()].clone().unwrap());
          models[index].material.occlusion_texture_strength = mat.occlusion_texture().map(|t| t.strength()).unwrap_or(0.0);
          models[index].material.emissive_texture = mat.emissive_texture().map(|t| gltf_textures[t.texture().index()].clone().unwrap());
          let emissive_factor = mat.emissive_factor();
          models[index].material.emissive_factor = Vector3::new(emissive_factor[0], emissive_factor[1], emissive_factor[2]);
          
          let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
          if let Some(iter) = reader.read_positions() {
            let mut vertices = Vec::with_capacity(iter.len());
            for vertex_position in iter {
              let vertex = Vector3::new(vertex_position[0], vertex_position[1], vertex_position[2]);
              let rotq = (quaternion*Quaternion::from_sv(0.0, vertex)*inverse_quaternion).v;
              let vertex = Vector4::new(rotq.x, rotq.y, rotq.z, 1.0);
              let vertex = (translation*scale)*vertex;
              vertices.push([vertex.x, vertex.y, vertex.z]);
            }
            models[index].vertices.vertex = vertices;
          }
          if let Some(iter) = reader.read_normals() {
            let mut normals = Vec::with_capacity(iter.len());
            for vertex_normal in iter {
              let normal = Vector4::new(vertex_normal[0], vertex_normal[1], vertex_normal[2], 1.0);
              let normal = (translation*scale)*normal;
              normals.push([normal.x, normal.y, normal.z]);
            }
            models[index].normals.normal = normals;
          }
          if let Some(iter) = reader.read_indices() {
            let mut indices = Vec::new();
            for vertex_indices in iter.into_u32() {
              indices.push(vertex_indices);
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
          index += 1;
        }
      }
    }
    //let mut materials: Vec<Material> = Vec::new();
    ModelDetails {
      models: models,
     // materials: materials,
    }
  }
  
  pub fn num_models(&self) -> usize {
    self.models.len()
  }
  
  pub fn vertex(&self, model_index: usize) -> Vec<[f32; 3]> {
    self.models[model_index].vertices.vertex.clone()
  }
  
  pub fn normal(&self, model_index: usize) -> Vec<[f32; 3]> {
    self.models[model_index].normals.normal.clone()
  }
  
  pub fn index(&self, model_index: usize) -> Vec<u32> {
    self.models[model_index].indices.index.clone()
  }
  
  pub fn texcoords(&self, model_index: usize) -> Vec<[f32; 2]> {
    self.models[model_index].texcoords.texcoord.clone()
  }
  
  pub fn base_colour(&self, model_index: usize) -> [f32; 4] {
    let colour = self.models[model_index].material.base_colour_factor;
    [colour.x, colour.y, colour.z, colour.w]
  }
  
  pub fn base_colour_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    self.models[model_index].material.base_colour_texture.clone()
  }
  
  pub fn metallic_factor(&self, model_index: usize) -> f32 {
    self.models[model_index].material.metallic_factor
  }
  
  pub fn roughness_factor(&self, model_index: usize) -> f32 {
    self.models[model_index].material.roughness_factor
  }
  
  pub fn metallic_roughness_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    self.models[model_index].material.metallic_roughness_texture.clone()
  }
  
  pub fn normal_texture_scale(&self, model_index: usize) -> f32 {
    self.models[model_index].material.normal_texture_scale
  }
  
  pub fn normal_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    self.models[model_index].material.normal_texture.clone()
  }
  
  pub fn occlusion_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    self.models[model_index].material.occlusion_texture.clone()
  }
  
  pub fn occlusion_texture_strength(&self, model_index: usize) -> f32 {
    self.models[model_index].material.occlusion_texture_strength
  }
  
  pub fn emissive_texture(&self, model_index: usize) -> Option<image::DynamicImage> {
    self.models[model_index].material.emissive_texture.clone()
  }
  
  pub fn emissive_factor(&self, model_index: usize) -> [f32; 3] {
    let emissive = self.models[model_index].material.emissive_factor;
    [emissive.x, emissive.y, emissive.z]
  }
  
  pub fn get_primitive_topology(&self, model_index: usize) -> Topology {
    self.models[model_index].topology.clone()
  }
}

fn texture_to_image(texture: gltf::Texture, buffers: &Vec<gltf::buffer::Data>, base_path: &Path) -> Option<image::DynamicImage> {
  let data = texture.source().source();
  let img = match data {
    gltf::image::Source::View { ref view, mime_type } => {
      let data = &buffers[view.offset()].to_vec();//buffers.view(view).expect("Failed to get buffer view for image");
      match mime_type {
        "image/jpeg" => image::load_from_memory_with_format(data, JPEG),
        "image/png" => image::load_from_memory_with_format(data, PNG),
        _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                    texture.index(), mime_type)),
      }
    },
    gltf::image::Source::Uri { uri, mime_type } => {
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
            
