use ThreadPool;

use gl;
use gl::types::*;

use image;

use std::mem;
use std::time;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
enum ObjectType {
  Texture(Option<GLuint>),
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
  data: Vec<Arc<Mutex<Option<(LoadableObject)>>>>,
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
  pub fn recieve_textures(&mut self) {
    if self.num_recv_objects <= 0 {
      self.data.clear();
      return;
    }
    
    let num = self.num_recv_objects;
    for _ in 0..num {
      match self.rx.try_recv() {
        Ok(i) => {
          let mut data = self.data[i].lock().unwrap();
          let object = data.take().unwrap();
          self.objects.push(object);
        },
        Err(e) => { },
      }
    }
  }
  
  /**
  ** Returns None when resource isnt loaded yet otherwise returns a ImmutableImage of format R8G8B8A8Unorm thats already in memory.
  **/
  pub fn get_texture(&mut self, reference: String) -> Option<GLuint> {
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
  pub fn insert_texture(&mut self, reference: String, new_image: GLuint) {
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
  pub fn sync_load_texture(&mut self, reference: String, location: String) {
    let texture = ResourceManager::load_texture_into_memory(reference.clone(), location.clone());
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: location.clone(),
        reference: reference.clone(),
        object_type: ObjectType::Texture(Some(texture)),
      }
    );
  }
  
  /**
  ** Loads textures in seperate threads, non bloacking. The function 
  **/
  pub fn load_texture(&mut self, reference: String, location: String) {
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
      let texture = ResourceManager::load_texture_into_memory(reference.clone(), location.clone());
      
      let object = LoadableObject {
        loaded: true,
        location: location,
        reference: reference,
        object_type: ObjectType::Texture(Some(texture)),
      };
      
      *data = Some(object);
      tx.send(index.clone()).unwrap();
    });
  }
  
  fn load_texture_into_memory(reference: String, location: String) -> GLuint {
    let texture_start_time = time::Instant::now();
    
    let mut texture_id: GLuint = 0;
    
    unsafe {
      gl::GenTextures(1, &mut texture_id);
      
      gl::BindTexture(gl::TEXTURE_2D, texture_id);
      
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
      
      let texture = location.clone();
      let image = image::open(&location).expect(&("No file or Directory at: ".to_string() + &texture)).to_rgba(); 
      let (width, height) = image.dimensions();
      let image_data = image.into_raw().clone();
     
      gl::TexImage2D(gl::TEXTURE_2D, 0,
                    gl::RGBA as GLint,
                    width as GLsizei,
                    height as GLsizei,
                    0, gl::RGBA, gl::UNSIGNED_BYTE,
                    mem::transmute(&image_data[0]));
      gl::GenerateMipmap(gl::TEXTURE_2D);
      
      gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
    
    texture_id
  }
}
