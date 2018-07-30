use std::{fs, io};
use std::path::Path;

use base64;

use gltf;
use gltf::material::AlphaMode;
use gltf::texture::MagFilter;
use gltf::texture::MinFilter;
use gltf::texture::WrappingMode;

//use gltf_importer;
//use gltf_importer::config::ValidationStrategy;

use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::Quaternion;
use cgmath::Rotation;

use image;
use image::ImageFormat::{JPEG, PNG};

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
struct TangentArray {
  tangent: Vec<[f32; 4]>,
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
struct ColourArray {
  colour: Vec<[f32; 4]>,
}

#[derive(Clone)]
pub struct Sampler {
  pub mag_filter: MagFilter,
  pub min_filter: MinFilter,
  pub wrap_s: WrappingMode,
  pub wrap_t: WrappingMode,
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
  base_colour_texture: Option<(Option<image::DynamicImage>, Sampler)>,
  metallic_factor: f32,
  roughness_factor: f32,
  metallic_roughness_texture: Option<(Option<image::DynamicImage>, Sampler)>,
  normal_texture_scale: f32,
  normal_texture: Option<(Option<image::DynamicImage>, Sampler)>,
  occlusion_texture: Option<(Option<image::DynamicImage>, Sampler)>,
  occlusion_texture_strength: f32,
  emissive_texture: Option<(Option<image::DynamicImage>, Sampler)>,
  emissive_factor: Vector3<f32>,
  alpha_mode: AlphaMode,
  alpha_cutoff: f32,
}

#[derive(Clone)]
struct FinalModel {
  vertices: VertexArray,
  indices: IndexArray,
  normals: NormalArray,
  tangents: TangentArray,
  texcoords: TexCoordArray,
  colours: ColourArray,
  material: Material,
  topology: Topology,
  has_indices: bool,
  has_normals: bool,
  has_tangents: bool,
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
     alpha_mode: AlphaMode::Blend,
     alpha_cutoff: 0.5,
   }
 }
}

impl Sampler {
  pub fn new() -> Sampler {
    Sampler {
      mag_filter: MagFilter::Linear,
      min_filter: MinFilter::Linear,
      wrap_s: WrappingMode::ClampToEdge,
      wrap_t: WrappingMode::ClampToEdge,
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
    //println!("{}", source);
    let load = gltf::import(source);
    let (gltf, buffers, images) = match load {
      Ok(t) => {
        t
      },
      Err(e) => {
        println!("{:?}", e);
        panic!();
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
  /*  let mut models = FinalModel {
      vertices: VertexArray,
      indices: IndexArray,
      normals: NormalArray,
      texcoords: TexCoordArray,
      material_ref: String,
      animation: Animation,
  }*/
    
    let gltf_textures_samplers: Vec<(Option<image::DynamicImage>, Sampler)> = {
      let mut textures_samplers = Vec::new();
      
      for texture in gltf.textures() {
        //println!("Texture: {:?}", texture.source().index());
        let img: Option<image::DynamicImage> = texture_to_image(texture.clone(), &buffers, &Path::new(&source));
        
        let texture_sampler = texture.sampler();
        let sampler = Sampler {
          mag_filter: texture_sampler.mag_filter().unwrap_or(MagFilter::Linear),
          min_filter: texture_sampler.min_filter().unwrap_or(MinFilter::Linear),
          wrap_s: texture_sampler.wrap_s(),
          wrap_t: texture_sampler.wrap_t(),
        };
        textures_samplers.push((img, sampler));
      }
      
      textures_samplers
    };
    
    let mut index = 0;
    for node in gltf.nodes() {
      let (translation, rotation, scale) = node.transform().decomposed();
      let scale = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
      let translation = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
      let quaternion = Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]);
      let inverse_quaternion = quaternion.invert();
      let rotation = Matrix4::from(quaternion);
      
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
              let vertex = Vector3::new(vertex_position[0], vertex_position[1], vertex_position[2]);
              //let rotq = (quaternion*Quaternion::from_sv(0.0, vertex)*inverse_quaternion).v;
              let vertex = (translation*scale)*rotation*Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);
              vertices.push([vertex.x, vertex.y, vertex.z]);
            }
            models[index].vertices.vertex = vertices;
          }
          if let Some(iter) = reader.read_normals() {
            let mut normals = Vec::with_capacity(iter.len());
            for vertex_normal in iter {
              let normal = Vector4::new(vertex_normal[0], vertex_normal[1], vertex_normal[2], 1.0);
              let normal = (translation*scale)*rotation*Vector4::new(normal.x, normal.y, normal.z, 1.0);
              normals.push([normal.x, normal.y, normal.z]);
              
              models[index].has_normals = true;
            }
            models[index].normals.normal = normals;
          }
          if let Some(iter) = reader.read_tangents() {
            let mut tangents = Vec::with_capacity(iter.len());
            for vertex_tangent in iter {
              let tangent = Vector4::new(vertex_tangent[0], vertex_tangent[1], vertex_tangent[2], vertex_tangent[3]);
              let normal = (translation*scale)*rotation*Vector4::new(tangent.x, tangent.y, tangent.z, tangent.w);
              tangents.push([tangent.x, tangent.y, tangent.z, tangent.w]);
              
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
     // materials: materials,
    }
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
  
  pub fn has_indices(&self, model_index: usize) -> bool {
    self.models[model_index].has_indices
  }
  
  pub fn has_normals(&self, model_index: usize) -> bool {
    self.models[model_index].has_normals
  }
  
  pub fn has_tangents(&self, model_index: usize) -> bool {
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
  
  pub fn base_colour_sampler(&self, model_index: usize) -> Sampler {
    let mut sampler = Sampler::new();
    if self.models[model_index].material.base_colour_texture.is_some() {
      sampler = self.models[model_index].material.base_colour_texture.clone().unwrap().1;
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
  
  pub fn metallic_roughness_sampler(&self, model_index: usize) -> Sampler {
    let mut sampler = Sampler::new();
    if self.models[model_index].material.metallic_roughness_texture.is_some() {
      sampler = self.models[model_index].material.metallic_roughness_texture.clone().unwrap().1;
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
  
  pub fn normal_sampler(&self, model_index: usize) -> Sampler {
    let mut sampler = Sampler::new();
    if self.models[model_index].material.normal_texture.is_some() {
      sampler = self.models[model_index].material.normal_texture.clone().unwrap().1
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
  
  pub fn occlusion_sampler(&self, model_index: usize) -> Sampler {
    let mut sampler = Sampler::new();
    if self.models[model_index].material.occlusion_texture.is_some() {
      sampler = self.models[model_index].material.occlusion_texture.clone().unwrap().1
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
  
  pub fn emissive_sampler(&self, model_index: usize) -> Sampler {
    let mut sampler = Sampler::new();
    if self.models[model_index].material.emissive_texture.is_some() {
      sampler = self.models[model_index].material.emissive_texture.clone().unwrap().1
    }
    sampler
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
            
