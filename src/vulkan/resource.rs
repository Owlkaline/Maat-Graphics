use ThreadPool;

use vulkano::format;
use vulkano::device::Queue;
use vulkano::sync::NowFuture;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::CommandBufferExecFuture;

use image;
use vulkano::image as vkimage;
use vulkano::image::ImmutableImage;

use std::time;
use std::sync::Arc;
use std::sync::mpsc;

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
  objects: Vec<LoadableObject>,
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
    if self.objects.len() > 0 {
      println!("More than 0");
      match self.objects[0].object_type {
        ObjectType::Texture(ref image) => {
          println!("{:?}", image.clone());
          image.clone()
        },
        _ => { 
          None 
        }
      }
    } else {
      None
    }
  }
  
  pub fn load_texture(&mut self, reference: String, location: String, queue: Arc<Queue>) {
    let mut object = LoadableObject {
      loaded: false,
      location: location.clone(),
      reference: reference.clone(),
      object_type: ObjectType::Texture(None),
    };
    
    let pos = self.objects.len();
    
    self.objects.push(object);
    
    let (tx, rx) = mpsc::channel();
    
    self.pool.execute(move || {
      let (texture, future) = ResourceManager::load_texture_into_memory(reference.clone(), location.clone(), queue);
      let object = LoadableObject {
        loaded: true,
        location: location,
        reference: reference,
        object_type: ObjectType::Texture(Some(texture)),
      };
      
      tx.send(object).unwrap();
    });
    
    let object = rx.recv().unwrap();
    self.objects[pos] = object;
    
  }
  
  fn load_texture_into_memory(reference: String, location: String, queue: Arc<Queue>) -> (Arc<ImmutableImage<format::R8G8B8A8Unorm>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    let texture_start_time = time::Instant::now();
    
    let (texture, tex_future) = {
      let texture = location.clone();
      let image = image::open(&location.clone()).expect(&("No file or Directory at: ".to_string() + &location)).to_rgba(); 
      let (width, height) = image.dimensions();
      let image_data = image.into_raw().clone();
      
      vkimage::immutable::ImmutableImage::from_iter(
              image_data.iter().cloned(),
              vkimage::Dimensions::Dim2d { width: width, height: height },
              format::R8G8B8A8Unorm,
              queue).unwrap()
    };
    /*
    objects.push(
      LoadableObject {
        loaded: true,
        location: location.clone(),
        reference: reference.clone(),
        object_type: ObjectType::Texture(Some(texture)),
      }
    );*/
    
    //let mut otype = *object.object_type;
//    otype = &mut ObjectType::Texture(Some(texture.clone()));
    //*otype = ObjectType::Texture(None);
    
    //self.previous_frame_end = Some(Box::new(tex_future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    //self.textures.insert(reference.clone(), texture);
    
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
    
    (texture, tex_future)
  }
}
