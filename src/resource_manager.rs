use crate::ThreadPool;

use vk;
use image;

use crate::vulkan::vkenums::{ImageType, ImageViewType, ImageTiling, SampleCount};

use crate::vulkan::{ImageAttachment, Instance, Device};
use crate::vulkan::buffer::{Buffer};
use crate::vulkan::pool::{CommandPool};

use crate::gltf_interpreter::ModelDetails;
use crate::font::GenericFont;

use crate::imgui::ImGui;

use cgmath::Vector3;

use std::time;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;

#[derive(Clone)]
enum ObjectType {
  Font(Option<(GenericFont, ImageAttachment)>),
  Texture(Option<image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>>, Option<ImageAttachment>),
  Model(Option<ModelDetails>, Vec<Option<ImageAttachment>>),
  _Shape(Option<(Buffer<f32>, ImageAttachment)>),
}

#[derive(Clone)]
struct LoadableObject {
  pub loaded: bool,
  pub location: String,
  pub reference: String,
  pub object_type: ObjectType,
}

impl LoadableObject {
  pub fn load_object(&mut self, instance: Arc<Instance>, device: Arc<Device>, image_type: &ImageType, image_view_type: &ImageViewType, format: &vk::Format, samples: &SampleCount, tiling: &ImageTiling, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
    let mut object;
    
    match &self.object_type {
      ObjectType::Texture(Some(image_data), ..) => {
        let buffer_image;
        let image = Some(ImageAttachment::create_texture(instance, device, image_data, image_type, tiling, samples, image_view_type, *format, command_pool, graphics_queue));
        
        buffer_image = image;
        object = ObjectType::Texture(None, buffer_image);
      },
      
      ObjectType::Model(Some(model), ..) => {
        let mut images = Vec::new();
        for i in 0..model.num_models() {
          if let Some(image_data) = model.base_colour_texture(i) {
            let image_data = image_data.to_rgba();
            let base_image = Some(ImageAttachment::create_texture(Arc::clone(&instance), Arc::clone(&device), &image_data, image_type, tiling, samples, image_view_type, *format, command_pool, graphics_queue));
            images.push(base_image);
          } else {
            images.push(None);
          }
          
          if let Some(image_data) = model.metallic_roughness_texture(i) {
            let image_data = image_data.to_rgba();
            let base_image = Some(ImageAttachment::create_texture(Arc::clone(&instance), Arc::clone(&device), &image_data, image_type, tiling, samples, image_view_type, *format, command_pool, graphics_queue));
            images.push(base_image);
          } else {
            images.push(None);
          }
          
          if let Some(image_data) = model.normal_texture(i) {
            let image_data = image_data.to_rgba();
            let base_image = Some(ImageAttachment::create_texture(Arc::clone(&instance), Arc::clone(&device), &image_data, image_type, tiling, samples, image_view_type, *format, command_pool, graphics_queue));
            images.push(base_image);
          } else {
            images.push(None);
          }
          
          if let Some(image_data) = model.occlusion_texture(i) {
            let image_data = image_data.to_rgba();
            let base_image = Some(ImageAttachment::create_texture(Arc::clone(&instance), Arc::clone(&device), &image_data, image_type, tiling, samples, image_view_type, *format, command_pool, graphics_queue));
            images.push(base_image);
          } else {
            images.push(None);
          }
          
          if let Some(image_data) = model.emissive_texture(i) {
            let image_data = image_data.to_rgba();
            let base_image = Some(ImageAttachment::create_texture(Arc::clone(&instance), Arc::clone(&device), &image_data, image_type, tiling, samples, image_view_type, *format, command_pool, graphics_queue));
            images.push(base_image);
          } else {
            images.push(None);
          }
        }
        
        object = ObjectType::Model(Some(model.clone()), images);
      },
      _ => { println!("No implemented to load yet"); return; },
    }
    
    self.loaded = true;
    self.object_type = object;
  }
}

pub struct ResourceManager {
  objects: Vec<LoadableObject>,
  pool: ThreadPool,
  num_recv_objects: i32,
  tx: mpsc::Sender<usize>,
  rx: mpsc::Receiver<usize>,
  data: Vec<Arc<Mutex<Option<(LoadableObject)>>>>,
}

impl ResourceManager {
  pub fn new() -> ResourceManager {
    let (tx, rx) = mpsc::channel();
    
    ResourceManager {
      objects: Vec::new(),
      pool: ThreadPool::new(50),
      num_recv_objects: 0,
      tx: tx,
      rx: rx,
      data: Vec::new(),
    }
  }
  
  pub fn pending_objects_loaded(&self) -> bool {
    let mut result = false;
    if self.data.len() == 0 {
      result = true;
    }
    result
  }
  
  /**
  ** Needs to be called frequently in backend to move resources from unknown land to somewhere where we can use it
  **/
  pub fn recieve_objects(&mut self, instance: Arc<Instance>, device: Arc<Device>, image_type: ImageType, image_view_type: ImageViewType, format: &vk::Format, samples: SampleCount, tiling: ImageTiling, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Vec<(String, Option<Vector3<f32>>)> {
    let mut references = Vec::new();
    
    if self.num_recv_objects <= 0 {
      if self.data.len() > 0 {
        self.data.clear();
      }
      return references;
    }
    
    let num = self.num_recv_objects;
    for _ in 0..num {
      match self.rx.try_recv() {
        Ok(i) => {
          let mut data = self.data[i].lock().unwrap();
          let mut object = data.take().unwrap();
          let reference = object.reference.to_string();
          
          object.load_object(Arc::clone(&instance), Arc::clone(&device), &image_type, &image_view_type, &format, &samples, &tiling, &command_pool, &graphics_queue);
          //println!("Object recieved: {}", object.reference);
          
          let mut size = None;
          match &object.object_type {
            ObjectType::Model(Some(model), ..) => {
              size = Some(model.get_size());
            }
            _ => {}
          }
          
          self.objects.push(object);
          
          references.push((reference, size));
          self.num_recv_objects -= 1;
        },
        Err(_e) => { },
      }
    }
    
    references
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    for object in &self.objects {
      match object {
        LoadableObject { loaded: true, location: _, reference: _, object_type } => {
          match object_type {
            ObjectType::Texture(_data, some_image) => {
              if let Some(image) = some_image {
                image.destroy(Arc::clone(&device));
              }
            },
            ObjectType::Font(some_image) => {
              if let Some((_font, image)) = some_image {
                image.destroy(Arc::clone(&device));
              }
            },
            ObjectType::_Shape(some_image) => {
              if let Some((_buffer, image)) = some_image {
                image.destroy(Arc::clone(&device));
              }
            },
            ObjectType::Model(_, images) => {
              for some_image in images {
                if let Some(image) = some_image {
                  image.destroy(Arc::clone(&device));
                }
              }
            },
          }
        },
        _ => {},
      }
    }
  }
  
  fn get_unloaded_object(&mut self, reference: String) -> Option<LoadableObject> {
    let mut object = None;
    
    for i in 0..self.objects.len() {
      if self.objects[i].reference == reference {
        if !self.objects[i].loaded {
          object = Some(self.objects.remove(i));
          break;
        }
      }
    }
    object
  }
  
  pub fn _remove_object(&mut self, reference: String) {
    for i in 0..self.objects.len() {
      if self.objects[i].reference == reference {
        self.objects.remove(i);
      }
    }
  }
  
  /**
  ** Returns None when resource isnt loaded yet otherwise returns a ImmutableImage of format R8G8B8A8Unorm thats already in memory.
  **/
  pub fn get_texture(&mut self, reference: String) -> Option<ImageAttachment> {
    let mut result = None;
    
    for object in self.objects.iter().rev() {
      if object.reference == reference {
        match object.object_type {
          ObjectType::Texture(ref _data, ref image) => {
            result = image.clone()
          },
          _ => {}
        }
      }
    }
    
    result
  }
  
  /**
  ** Returns None when resource isnt loaded yet otherwise returns a ModelDetails
  **/
  pub fn get_model(&mut self, reference: String) -> Option<(Option<ModelDetails>, Vec<Option<ImageAttachment>>)> {
    let mut result = None;
    
    for object in &self.objects {
      if object.reference == reference {
        match object.object_type {
          ObjectType::Model(ref model, ref images) => {
            result = Some((model.clone(), images.clone()));
          },
          _ => {}
        }
      }
    }
    
    result
  }
  
  /**
  ** Returns None when resource isnt loaded yet otherwise returns font thats already in memory.
  **/
  pub fn get_font(&mut self, reference: String) -> Option<(GenericFont, ImageAttachment)> {
    let mut result: Option<(GenericFont, ImageAttachment)> = None;
    
    for object in &self.objects {
      if object.reference == reference {
        match object.object_type {
          ObjectType::Font(ref some_font_object) => {
            if let Some(font_object) = some_font_object {
              result = Some(font_object.clone());
            }
          },
          _ => {}
        }
      }
    }
    
    result
  }
  
  pub fn get_all_textures(&self) -> Vec<(String, ImageAttachment)> {
    let mut result = Vec::with_capacity(self.objects.len());
    
    for object in &self.objects {
      if !object.loaded { continue; }
      let reference = object.reference.to_string();
      match object.object_type {
        ObjectType::Texture(ref _data, ref image) => {
          if image.is_some() {
            result.push((reference, image.clone().unwrap()));
          }
        },
        _ => {}
      }
    }
    
    result
  }
  
  /**
  ** Inserts details for a texture, does not load the image into memory.
  ** Must call Load_texture as a DrawCall in order to use
  **/
  pub fn insert_unloaded_texture(&mut self, reference: String, location: String) {
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
   // println!("Inserting object: {}", reference);
    self.objects.push(
      LoadableObject {
        loaded: false,
        location: location,
        reference: reference.clone(),
        object_type: ObjectType::Texture(None, None),
      }
    );
  }
  
  /**
  ** Inserts a image that was created elsewhere in the program into the resource manager, a location is not required here as it is presumed that it was not created from a file that the ResourceManager has access to.
  **/
  pub fn insert_texture(&mut self, reference: String, new_image: ImageAttachment) {
   // println!("inserting texture");
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Texture(None, Some(new_image)),
      }
    );
  }
  
  /**
  ** Forces thread to wait until resource is loaded into memory.
  **/
  pub fn sync_load_texture(&mut self, reference: String, location: String, device: Arc<Device>, instance: Arc<Instance>, command_pool: &CommandPool, queue: vk::Queue) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
  
    let texture = ResourceManager::load_texture_into_memory(location.clone(), instance, device, command_pool, queue);
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: location.clone(),
        reference: reference.clone(),
        object_type: ObjectType::Texture(None, Some(texture)),
      }
    );
  }
  
  /**
  ** Loads textures from inserted details in seperate threads, non bloacking.
  **/
  pub fn load_texture_from_reference(&mut self, reference: String) {
   // debug_assert!(!self.check_object(reference.clone()), "Error: Object {} doesn't exist!", reference);
    
    let unloaded_object = self.get_unloaded_object(reference.clone());
    if let Some(object) = unloaded_object {
      let location = object.location;
      let reference = object.reference;
      
      self.load_texture(reference, location);
    } else {
    //  println!("Object {} already loaded", reference);
    }
  }
  
  /**
  ** Unloads textures.
  **/
  pub fn _unload_texture_from_reference(&mut self, device: Arc<Device>, reference: String) {
   // debug_assert!(!self.check_object(reference.clone()), "Error: Object {} doesn't exist!", reference);
    for i in 0..self.objects.len() {
      if self.objects[i].reference == reference {
        match &self.objects[i].object_type {
          ObjectType::Texture(_data, some_image) => {
            if let Some(image) = some_image {
              image.destroy(Arc::clone(&device));
            }
            self.objects.remove(i);
            break;
          },
          _ => {}
        }
      }
    }
  }
  
  /**
  ** Unloads models.
  **/
  pub fn unload_model_from_reference(&mut self, device: Arc<Device>, reference: String) {
    for i in 0..self.objects.len() {
      let mut object_index: i32 = -1;
      if self.objects[i].reference == reference {
        match &self.objects[i].object_type {
ObjectType::Model(_, images) => {
          object_index = i as i32;
          for some_image in images {
            if let Some(image) = some_image {
              image.destroy(Arc::clone(&device));
            }
          }
        }
        _ => {}
        }
      }
      if object_index > -1 {
        self.objects[object_index as usize].loaded = false;
      }
    }
  }
  
  
  /**
  ** Only way to laod new font, Forces thread to wait until resource is loaded into memory.
  **/
  pub fn sync_load_font(&mut self, reference: String, location: String, font: &[u8], device: Arc<Device>, instance: Arc<Instance>, command_pool: &CommandPool, queue: vk::Queue) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    let texture = ResourceManager::load_texture_into_memory(location.clone(), instance, device, command_pool, queue);
    let font = ResourceManager::load_font_into_memory(reference.clone(), font);
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: location.clone(),
        reference: reference.clone(),
        object_type: ObjectType::Font(Some((font, texture))),
      }
    );
  }
  
  fn load_font_into_memory(reference: String, font: &[u8]) -> GenericFont {
    let font_start_time = time::Instant::now();
    
    let mut new_font = GenericFont::new();
    new_font.load_font(font);
    
    let font_time = font_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms, Font: {:?}", (font_time*1000f64) as f32, reference);
    
    new_font
  }
  
  
  /**
  ** Inserts details for a model, does not load the image into memory.
  ** Must call Load_model as a DrawCall in order to use
  **/
  pub fn insert_unloaded_model(&mut self, reference: String, location: String) {
   
   if !self.check_object(reference.clone()) {
     return;
   }
   
    //println!("Inserting object: {}", reference);
    self.objects.push(
      LoadableObject {
        loaded: false,
        location: location,
        reference: reference.clone(),
        object_type: ObjectType::Model(None, Vec::new()),
      }
    );
  }
  
  /**
  ** Loads models from inserted details in seperate threads, non bloacking.
  **/
  pub fn load_model_from_reference(&mut self, reference: String) {
    let unloaded_object = self.get_unloaded_object(reference.clone());
    if let Some(object) = unloaded_object {
      let location = object.location;
      let reference = object.reference;
      
      self.load_model(reference, location);
    } else {
      //println!("Object {} already loaded", reference);
    }
  }
  
  pub fn load_imgui(&mut self, instance: Arc<Instance>, device: Arc<Device>, imgui: &mut ImGui, command_pool: &CommandPool, graphics_queue: vk::Queue) {
    let mut fonts = imgui.fonts();
    let texture = fonts.build_rgba32_texture();
    let data = texture.data;
    let raw_pixels = data.iter().map(|p| *p as u8).collect::<Vec<u8>>();
    let texture = ImageAttachment::create_texture_from_pixels(Arc::clone(&instance), Arc::clone(&device), raw_pixels, texture.width, texture.height, &ImageType::Type2D, &ImageTiling::Optimal, &SampleCount::OneBit, &ImageViewType::Type2D, vk::FORMAT_R8G8B8A8_UNORM, command_pool, &graphics_queue);
    self.insert_texture("imgui".to_string(), texture);
  }
  
  /**
  ** Loads textures in seperate threads, non bloacking.
  **/
  pub fn load_texture(&mut self, reference: String, location: String) {
    
    debug_assert!(self.check_object(reference.clone()), "Error: Object reference already exists!");
    //println!("loading texture");
    self.num_recv_objects += 1;
    let index = self.data.len();
    
    self.data.push(Arc::new(Mutex::new(None)));
    
    let (data, tx) = (self.data[index].clone(), self.tx.clone());
    self.pool.execute(move || {
      let mut data = data.lock().unwrap();
      let texture_start_time = time::Instant::now();
      let texture = image::open(&location.clone()).expect(&("No file or Directory at: ".to_string() + &location)).to_rgba();
     // println!("Texture is loading: {}", location);
   //   if location.to_string() == "./resources/Textures/Logo.png".to_string() {
       // println!("{:?}", texture);
    //  }
      
      let object = LoadableObject {
        loaded: true,
        location: location.to_string(),
        reference: reference,
        object_type: ObjectType::Texture(Some(texture), None),
      };
      
      let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
      
      *data = Some(object);
      tx.send(index.clone()).unwrap();
    });
  }
  
  fn load_texture_into_memory(location: String, instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: vk::Queue) -> (ImageAttachment) {
    let texture_start_time = time::Instant::now();
    
    let texture = ImageAttachment::create_texture_from_location(instance, device, location.to_string(), &ImageType::Type2D, &ImageTiling::Optimal, &SampleCount::OneBit, &ImageViewType::Type2D, vk::FORMAT_R8G8B8A8_UNORM, command_pool, &graphics_queue);
    
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
    
    (texture)
  }
  
  fn check_object(&self, reference: String) -> bool {
    let mut result = true;
    for object in &self.objects {
      if object.reference == reference {
        result = false;
      }
    }
    result
  }
  
  /**
  ** Loads models in the main thread blocking.
  **/
  pub fn sync_load_model(&mut self, reference: String, location: String) {
    
    debug_assert!(self.check_object(reference.clone()), "Error: Object reference already exists!");
    
   // println!("loading model");
    
    let model_start_time = time::Instant::now();
    let model = ModelDetails::new(location.to_string());
      
    let object = LoadableObject {
      loaded: true,
      location: location.to_string(),
      reference: reference,
      object_type: ObjectType::Model(Some(model), Vec::new()),
    };
    
    let model_time = model_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (model_time*1000f64) as f32, location);
    
    self.objects.push(object);
  }
  
  /**
  ** Loads modelss in seperate threads, non bloacking.
  **/
  pub fn load_model(&mut self, reference: String, location: String) {
    
    debug_assert!(self.check_object(reference.clone()), "Error: Object reference already exists!");
    //println!("loading model");
    self.num_recv_objects += 1;
    let index = self.data.len();
    
    self.data.push(Arc::new(Mutex::new(None)));
    
    let (data, tx) = (self.data[index].clone(), self.tx.clone());
    self.pool.execute(move || {
      let mut data = data.lock().unwrap();
      let model_start_time = time::Instant::now();
      let model = ModelDetails::new(location.to_string());
      
      let object = LoadableObject {
        loaded: true,
        location: location.to_string(),
        reference: reference,
        object_type: ObjectType::Model(Some(model), Vec::new()),
      };
      
      let model_time = model_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      println!("{} ms,  {:?}", (model_time*1000f64) as f32, location);
      
      *data = Some(object);
      tx.send(index.clone()).unwrap();
    });
  }
  
  /*
  /**
  ** Returns None when resource isnt loaded yet otherwise returns Vertex and Index buffers thats already in memory.
  **/
  pub fn get_shape(&mut self, reference: String) -> Option<(Arc<BufferAccess + Send + Sync>, Arc<ImmutableBuffer<[u32]>>)> {
    let mut result = None;
    
    for object in &self.objects {
      if object.reference == reference {
        match object.object_type {
          ObjectType::Shape(ref buffer) => {
            result = buffer.clone()
          },
          _ => {}
        }
      }
    }
    
    result
  }
  
  /**
  ** Inserts a shape (vertex + index) that was created elsewhere in the program into the resource manager
  **/
  pub fn insert_shape(&mut self, reference: String, shape_info: (Arc<BufferAccess + Send + Sync>, Arc<ImmutableBuffer<[u32]>>)) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Shape(Some(shape_info)),
      }
    );
  }
  
  /**
  ** Forces thread to wait until resource is loaded into memory.
  **/
  pub fn sync_load_shape(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>, queue: Arc<Queue>) -> Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>> {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    let (vertex, index, futures) = ResourceManager::load_shape_into_memory(reference.clone(), vertex, index, queue);
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Shape(Some((vertex, index))),
      }
    );
    
    futures
  }
  
  /**
  ** Loads vertex and index in a seperate thread, non bloacking.
  **/
  pub fn load_shape(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>, queue: Arc<Queue>) {
    println!("loading shape");
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    self.num_recv_objects += 1;
    let idx = self.data.len();
    
    self.data.push(Arc::new(Mutex::new(None)));
    
    let (data, tx) = (self.data[idx].clone(), self.tx.clone());
    self.pool.execute(move || {
      let mut data = data.lock().unwrap();
      let (vertex, index, futures) = ResourceManager::load_shape_into_memory(reference.clone(), vertex, index, queue);
      
      let object = LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference,
        object_type: ObjectType::Shape(Some((vertex, index))),
      };
      
      *data = Some((object, futures));
      tx.send(idx.clone()).unwrap();
    });
  }
  
  pub fn update_shape(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>, queue: Arc<Queue>) -> Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>> {
    println!("updating shape");
    let (vertex, index, futures) = ResourceManager::load_shape_into_memory(reference.clone(), vertex, index, queue);
    
    let mut found = false;
    
    for i in 0..self.objects.len() {
      if self.objects[i].reference == reference {
        self.objects[i].object_type = ObjectType::Shape(Some((Arc::clone(&vertex), Arc::clone(&index))));
        found = true;
        break;
      }
    }
    
    if !found {
      self.insert_shape(reference, (vertex, index));
    }
    
    futures
  }
  
  
  fn load_shape_into_memory(reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>, queue: Arc<Queue>) -> (Arc<BufferAccess + Send + Sync>, Arc<ImmutableBuffer<[u32]>>, Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>) {
    let shape_start_time = time::Instant::now();
    
    let (vertex, future_vtx) = ImmutableBuffer::from_iter(vertex.iter().cloned(),
                                                          BufferUsage::vertex_buffer(),
                                                          Arc::clone(&queue))
                                                          .expect("failed to create immutable vertex buffer");
                               
    let (index, future_idx) = ImmutableBuffer::from_iter(index.iter().cloned(),
                                                         BufferUsage::index_buffer(),
                                                         queue)
                                                         .expect("failed to create immutable index buffer");
    
    let shape_time = shape_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms, Shape: {:?}", (shape_time*1000f64) as f32, reference);
    
    (vertex, index, vec!(future_vtx, future_idx))
  }
  
  */
}
