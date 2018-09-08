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
use std::sync::Mutex;

#[derive(Clone, Debug)]
enum ObjectType {
  Texture(Option<Arc<ImmutableImage<format::R8G8B8A8Unorm>>>),
  Model(String),
}

#[derive(Clone, Debug)]
struct LoadableObject {
  pub loaded: bool,
  pub location: String,
  pub reference: String,
  pub object_type: ObjectType,
}

pub struct ResourceManager {
  objects: Vec<LoadableObject>,
  pool: ThreadPool,
  num_recv_objects: i32,
  tx: mpsc::Sender<usize>,
  rx: mpsc::Receiver<usize>,
  data: Vec<Arc<Mutex<Option<(LoadableObject, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>)>>>>,
}

impl ResourceManager {
  pub fn new() -> ResourceManager {
    let (tx, rx) = mpsc::channel();
    
    ResourceManager {
      objects: Vec::new(),
      pool: ThreadPool::new(10),
      num_recv_objects: 0,
      tx: tx,
      rx: rx,
      data: Vec::new(),
    }
  }
  
  /**
  ** Needs to be called frequently in backend to move resources from unknown land to somewhere where we can use it
  **/
  pub fn recieve_textures(&mut self) -> Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>> {
    let mut futures = Vec::new();
    
    if self.num_recv_objects <= 0 {
      self.data.clear();
      return futures;
    }
    
    let num = self.num_recv_objects;
    for _ in 0..num {
      match self.rx.try_recv() {
        Ok(i) => {
          let mut data = self.data[i].lock().unwrap();
          let (object, future) = data.take().unwrap();
          self.objects.push(object);
          futures.push(future);
        },
        Err(e) => { },
      }
    }
    
    futures
  }
              //    self.objects.push(object);
              //  futures.push(future);
              
  /**
  ** Returns None when resource isnt loaded yet otherwise returns a ImmutableImage of format R8G8B8A8Unorm thats already in memory.
  **/
  pub fn get_texture(&mut self, reference: String) -> Option<Arc<ImmutableImage<format::R8G8B8A8Unorm>>> {
    let mut result = None;
    
    for object in &self.objects {
      if object.reference == reference {
        match object.object_type {
          ObjectType::Texture(ref image) => {
            result = image.clone()
          },
          _ => {}
        }
      }
    }
    
    result
  }
  
  /**
  ** Inserts a image that was created elsewhere in the program into the resource manager, a location is not required here as it is presumed that it was not created from a file that the ResourceManager has access to.
  **/
  pub fn insert_texture(&mut self, reference: String, new_image: Arc<ImmutableImage<format::R8G8B8A8Unorm>>) {
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Texture(Some(new_image)),
      }
    );
  }
  
  /**
  ** Forces thread to wait until resource is loaded into memory.
  **/
  pub fn sync_load_texture(&mut self, reference: String, location: String, queue: Arc<Queue>) -> CommandBufferExecFuture<NowFuture, AutoCommandBuffer> {
    let (texture, future) = ResourceManager::load_texture_into_memory(reference.clone(), location.clone(), queue);
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: location.clone(),
        reference: reference.clone(),
        object_type: ObjectType::Texture(Some(texture)),
      }
    );
    
    future
  }
  
  /**
  ** Loads textures in seperate threads, non bloacking. The function 
  **/
  pub fn load_texture(&mut self, reference: String, location: String, queue: Arc<Queue>) {
    let mut object = LoadableObject {
      loaded: false,
      location: location.clone(),
      reference: reference.clone(),
      object_type: ObjectType::Texture(None),
    };
    
    self.num_recv_objects += 1;
    let index = self.data.len();
    
    self.data.push(Arc::new(Mutex::new(None)));
    
    let (data, tx) = (self.data[index].clone(), self.tx.clone());
    self.pool.execute(move || {
      let mut data = data.lock().unwrap();
      let (texture, future) = ResourceManager::load_texture_into_memory(reference.clone(), location.clone(), queue);
      
      let object = LoadableObject {
        loaded: true,
        location: location,
        reference: reference,
        object_type: ObjectType::Texture(Some(texture)),
      };
      
      *data = Some((object, future));
      tx.send(index.clone()).unwrap();
      //self.tx.lock().unwrap().clone().send((object, future)).unwrap();
    });
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
    
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
    
    (texture, tex_future)
  }
}
