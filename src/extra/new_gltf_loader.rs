use std::collections::HashMap;

pub fn texture_label(texture: &gltf::Texture) -> String {
  format!("Texture{}", texture.index())
}

pub fn material_label(material: &gltf::Material) -> String {
  if let Some(index) = material.index() {
    format!("Material{}", index)
  } else {
    "MaterialDefault".to_string()
  }
}

struct Material {
  base_colour: [f32; 4],
  base_colour_texture: Option<String>,
  roughness: f32,
  metallic: f32,
  metallic_roughness_texture: Option<String>,
  normal_map: Option<String>,
  double_sided: bool,
  occlusion_texture: Option<String>,
  emissive: [f32; 3],
  emissive_texture: Option<String>,
}

impl Material {
  pub fn load_material(material: &gltf::Material) -> Material {
    let label = material_label(material);

    let pbr = material.pbr_metallic_roughness();

    let base_colour = pbr.base_color_factor();
    let base_colour_texture = if let Some(info) = pbr.base_color_texture() {
      let label = texture_label(&info.texture());
      Some(label)
    } else {
      None
    };

    let normal_map = if let Some(normal_texture) = material.normal_texture() {
      let label = texture_label(&normal_texture.texture());
      Some(label)
    } else {
      None
    };

    let metallic_roughness_texture = if let Some(info) = pbr.metallic_roughness_texture() {
      let label = texture_label(&info.texture());
      Some(label)
    } else {
      None
    };

    let occlusion_texture = if let Some(occlusion_texture) = material.occlusion_texture() {
      let label = texture_label(&occlusion_texture.texture());
      Some(label)
    } else {
      None
    };

    let emissive = material.emissive_factor();
    let emissive_texture = if let Some(info) = material.emissive_texture() {
      let label = texture_label(&info.texture());
      Some(label)
    } else {
      None
    };

    Material {
      base_colour,
      base_colour_texture,
      roughness: pbr.roughness_factor(),
      metallic: pbr.metallic_factor(),
      metallic_roughness_texture,
      normal_map,
      double_sided: material.double_sided(),
      occlusion_texture,
      emissive,
      emissive_texture,
    }
  }
}

pub fn new_load_glb(location: String) {
  let gltf = gltf::Gltf::open(location).unwrap();

  let mut buffer_data: Vec<Vec<u8>> = Vec::new();

  let mut materials = Vec::new();

  for buffer in gltf.buffers() {
    match buffer.source() {
      gltf::buffer::Source::Uri(uri) => {
        println!("Uri: {}", uri);
      }
      gltf::buffer::Source::Bin => {
        println!("Bin");
        if let Some(blob) = gltf.blob.as_deref() {
          buffer_data.push(blob.into());
        }
      }
    }
  }

  let mut named_materials = HashMap::new();

  for material in gltf.materials() {
    let material_data = Material::load_material(&material);
    if let Some(name) = material.name() {
      named_materials.insert(name.to_string(), material_data);
    }
    materials.push(material_data);
  }

  let meshes = Vec::new();
  let named_meshes = HashMap::new();

  for mesh in gltf.meshes() {
    let mut primitives = Vec::new();
    for primitive in mesh.primitives() {}
  }
}
