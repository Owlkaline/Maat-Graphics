use ThreadPool;

use vulkano::format;
use vulkano::device::Queue;

use image;
use vulkano::image as vkimage;
use vulkano::image::ImmutableImage;

use std::time;
use std::sync::Arc;

#[derive(Clone)]
enum ObjectType {
  Texture(Option<Arc<ImmutableImage<format::R8G8B8A8Unorm>>>),
  Model(String),
}

#[derive(Clone)]
struct LoadableObject {
  pub loaded: bool,
  pub location: String,
  pub reference: String,
  pub object_type: ObjectType,
}

pub struct ResourceManager {
  objects: Vec<Arc<LoadableObject>>,
  pool: ThreadPool,
}

impl ResourceManager {
  pub fn new() -> ResourceManager {
    ResourceManager {
      objects: Vec::new(),
      pool: ThreadPool::new(10),
    }
  }
  
  pub fn get_texture(&mut self) -> Option<Arc<ImmutableImage<format::R8G8B8A8Unorm>>> {
    match self.objects[0].object_type {
      ObjectType::Texture(ref image) => {
        image.clone()
      },
      _ => { 
        None 
      }
    }
  }
  
  pub fn load_texture(&mut self, reference: String, location: String, queue: Arc<Queue>) {
    let mut object = Arc::new(LoadableObject {
      loaded: false,
      location: location,
      reference: reference,
      object_type: ObjectType::Texture(None),
    });
    
    self.objects.push(object.clone());
    
    self.pool.execute(move || {
      ResourceManager::load_texture_into_memory(object, queue);
    });
    
  }
  
  fn load_texture_into_memory(object: Arc<LoadableObject>, queue: Arc<Queue>) {
    let texture_start_time = time::Instant::now();
    
    let location = &object.location;
    
    let (texture, tex_future) = {
      let texture = location.clone();
      let image = image::open(location).expect(&("No file or Directory at: ".to_string() + location)).to_rgba(); 
      let (width, height) = image.dimensions();
      let image_data = image.into_raw().clone();
      
      vkimage::immutable::ImmutableImage::from_iter(
              image_data.iter().cloned(),
              vkimage::Dimensions::Dim2d { width: width, height: height },
              format::R8G8B8A8Unorm,
              queue).unwrap()
    };
    
    let mut otype = *object.object_type;
//    otype = &mut ObjectType::Texture(Some(texture.clone()));
    *otype = ObjectType::Texture(None);
    
    //self.previous_frame_end = Some(Box::new(tex_future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    //self.textures.insert(reference.clone(), texture);
    
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
  }
}
