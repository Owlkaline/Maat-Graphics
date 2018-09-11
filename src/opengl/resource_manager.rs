use ThreadPool;

use graphics::Vertex2d;
use opengl::Vao;

use gl;
use gl::types::*;

use font::GenericFont;

use image;

use cgmath::Matrix4;

use std::mem;
use std::time;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;

#[derive(Clone)]
enum ObjectType {
  Font(Option<(GenericFont, GLuint)>),
  Texture(Option<GLuint>),
  Model(String),
  Shape(Option<(Vao)>),
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
  pub fn recieve_objects(&mut self) {
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
  
  pub fn remove_object(&mut self, reference: String) {
    for i in 0..self.objects.len() {
      if self.objects[i].reference == reference {
        self.objects.remove(i);
      }
    }
  }
  
  /**
  ** Returns None when resource isnt loaded yet otherwise returns Vertex and Index buffers thats already in memory.
  **/
  pub fn get_shape(&mut self, reference: String) -> Option<Vao> {
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
  pub fn insert_shape(&mut self, reference: String,  vertex: Vec<Vertex2d>, index: Vec<u32>) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    let vao = ResourceManager::load_shape_into_memory(reference.clone(), vertex, index);
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Shape(Some(vao)),
      }
    );
  }
  
  /**
  ** Forces thread to wait until resource is loaded into memory.
  **/
  pub fn sync_load_shape(&mut self, reference: String, location: String, vertex: Vec<Vertex2d>, index: Vec<u32>) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    let vao = ResourceManager::load_shape_into_memory(reference.clone(), vertex, index);
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Shape(Some(vao)),
      }
    );
  }
  
  /**
  ** Loads vertex and index in a seperate thread, non bloacking.
  **/
  pub fn load_shape(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    let object = LoadableObject {
      loaded: false,
      location: "".to_string(),
      reference: reference.clone(),
      object_type: ObjectType::Shape(None),
    };
    
    self.num_recv_objects += 1;
    let idx = self.data.len();
    
    self.data.push(Arc::new(Mutex::new(None)));
    
    let (data, tx) = (self.data[idx].clone(), self.tx.clone());
    self.pool.execute(move || {
      let mut data = data.lock().unwrap();
      let vao = ResourceManager::load_shape_into_memory(reference.clone(), vertex, index);
      
      let object = LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference,
        object_type: ObjectType::Shape(Some(vao)),
      };
      
      *data = Some(object);
      tx.send(idx.clone()).unwrap();
    });
  }
  
  pub fn update_shape(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>) {
    let mut verts = Vec::new();
    
    for v in vertex.clone() {
      verts.push(v.position[0] as GLfloat);
      verts.push(v.position[1] as GLfloat);
      verts.push(v.uv[0] as GLfloat);
      verts.push(v.uv[1] as GLfloat);
    };
    
    let index = index.iter().map(|i| {
      *i as GLuint
    }).collect::<Vec<GLuint>>();
    
    let mut found = false;
    
    for i in 0..self.objects.len() {
      if self.objects[i].reference == reference {
        match self.objects[i].object_type {
          ObjectType::Shape(ref mut vao) => {
            if let Some(ref mut vao) = vao {
              vao.update_vbo(verts);
              vao.update_ebo(index.clone());
              found = true;
              break;
            }
          },
          _ => {},
        }
      }
    }
    
    if !found {
      self.insert_shape(reference, vertex, index);
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
  
  /**
  ** Returns None when resource isnt loaded yet otherwise returns font thats already in memory.
  **/
  pub fn get_font(&mut self, reference: String) -> Option<(GenericFont, GLuint)> {
    let mut result = None;
    
    for object in &self.objects {
      if object.reference == reference {
        match object.object_type {
          ObjectType::Font(ref font) => {
            result = font.clone()
          },
          _ => {}
        }
      }
    }
    
    result
  }
  
  /**
  ** Inserts a font (GenericFont + Texture) that was created elsewhere in the program into the resource manager
  **/
  pub fn insert_font(&mut self, reference: String, font_info: (GenericFont, GLuint)) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    self.objects.push(
      LoadableObject {
        loaded: true,
        location: "".to_string(),
        reference: reference.clone(),
        object_type: ObjectType::Font(Some(font_info)),
      }
    );
  }
  
  /**
  ** Only way to laod new font, Forces thread to wait until resource is loaded into memory.
  **/
  pub fn sync_load_font(&mut self, reference: String, location: String, font: &[u8]) {
    
    debug_assert!(self.check_object(reference.clone()), "Error, Object reference already exists!");
    
    let texture = ResourceManager::load_texture_into_memory(reference.clone(), location.clone());
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
  
  fn check_object(&self, reference: String) -> bool {
    let mut result = true;
    for object in &self.objects {
      if object.reference == reference {
        result = false;
      }
    }
    result
  }
  
  fn load_font_into_memory(reference: String, font: &[u8]) -> GenericFont {
    let font_start_time = time::Instant::now();
    
    let mut new_font = GenericFont::new();
    new_font.load_font(font);
    
    let font_time = font_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms, Font: {:?}", (font_time*1000f64) as f32, reference);
    
    new_font
  }
  
  fn load_shape_into_memory(reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>) -> Vao {
    let shape_start_time = time::Instant::now();
    
    let mut verts = Vec::new();
    
    for v in vertex {
      verts.push(v.position[0] as GLfloat);
      verts.push(v.position[1] as GLfloat);
      verts.push(v.uv[0] as GLfloat);
      verts.push(v.uv[1] as GLfloat);
    };
    
    let mut vao = Vao::new();
    vao.bind();
    
    vao.create_ebo(index, gl::DYNAMIC_DRAW);
    vao.create_vbo(verts, gl::DYNAMIC_DRAW);
    
    vao.set_vertex_attrib(0, 2, 4, 0);
    vao.set_vertex_attrib(1, 2, 4, 2);
    
    let shape_time = shape_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms, Shape: {:?}", (shape_time*1000f64) as f32, reference);
    
    vao
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
