use vulkano::memory;
use vulkano::format;
use vulkano::sync::NowFuture;
use vulkano::image as vkimage;
use vulkano::image::ImmutableImage;
use vulkano::device::{Device, Queue};
use vulkano::buffer::{CpuBufferPool, cpu_pool,
                      BufferUsage, BufferAccess,
                      ImmutableBuffer, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, CommandBufferExecFuture};

use image;
use image::DynamicImage::ImageRgba8;

use graphics::Vertex3d;
use drawcalls::DrawCall;
use vulkan::rawvk::{vs_3d};

use cgmath::{Deg, perspective, Vector4, Matrix4};

use std::sync::Arc;
use std::iter;
use std::slice;

pub fn create_3d_projection(width: f32, height: f32) -> Matrix4<f32> {
  perspective(Deg(45.0), { width as f32 / height as f32 }, 0.1, 100.0)
}

pub fn create_vertex(device: Arc<Device>, verticies: iter::Cloned<slice::Iter<Vertex3d>>) -> Arc<BufferAccess + Send + Sync> {
    CpuAccessibleBuffer::from_iter(device, 
                                   BufferUsage::vertex_buffer(), 
                                   verticies)
                                   .expect("failed to create vertex buffer")
}

pub fn create_index(queue: Arc<Queue>, indices: iter::Cloned<slice::Iter<u32>>) -> (Arc<ImmutableBuffer<[u32]>>,
                                                                         CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    ImmutableBuffer::from_iter(indices, BufferUsage::index_buffer(), 
                               queue)
                               .expect("failed to create immutable teapot index buffer")
}

pub fn create_depth_buffer(device: Arc<Device>, dimensions: [u32; 2]) -> Option<Arc<vkimage::AttachmentImage<format::D16Unorm>>> {
  Some(vkimage::attachment::AttachmentImage::transient(
                              device,
                              dimensions,
                              format::D16Unorm)
                              .unwrap())
  }

pub fn create_texture_from_dynamicimage(queue: Arc<Queue>, data: Option<image::DynamicImage>) -> Option<(Arc<ImmutableImage<format::R8G8B8A8Unorm>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>)> {
//  mesh_data.base_colour_texture(i)
  let mut final_texture = None;
  if let Some(ImageRgba8(texture_img)) = data {
    let dim = texture_img.dimensions();
    let image_data = texture_img.into_raw().clone();
    
    final_texture = Some(vkimage::immutable::ImmutableImage::from_iter(
            image_data.iter().cloned(),
            vkimage::Dimensions::Dim2d { width: dim.0, height: dim.1 },
            format::R8G8B8A8Unorm,
            queue).unwrap());
  }
  
  final_texture
}

pub fn create_3d_subbuffer(draw: DrawCall, projection: Matrix4<f32>, view_matrix: Matrix4<f32>, uniform_buffer: CpuBufferPool<vs_3d::ty::Data>) -> cpu_pool::CpuBufferPoolSubbuffer<vs_3d::ty::Data, Arc<memory::pool::StdMemoryPool>> {
  
  let rotation_x: Matrix4<f32> = Matrix4::from_angle_x(Deg(draw.get_x_rotation()));
  let rotation_y: Matrix4<f32> = Matrix4::from_angle_y(Deg(draw.get_y_rotation()));
  let rotation_z: Matrix4<f32> = Matrix4::from_angle_z(Deg(draw.get_z_rotation()));
  
  let transformation: Matrix4<f32> = (Matrix4::from_translation(draw.get_translation())* Matrix4::from_scale(draw.get_size().x)) * (rotation_x*rotation_y*rotation_z);
  
  let point_light = 2.0;
  let directional_light = 0.0;
  let metallic = 1.0;
  let roughness = 1.0;
  
  let lighting_position: Matrix4<f32> =
    Matrix4::from_cols(
      // (x, y, z, n/a)
      Vector4::new(0.0, -0.6, 25.0, -1.0),
      Vector4::new(7.0, -0.6, 25.0, -1.0),
      Vector4::new(-2000000.0, 1000000.0, -2000000.0, -1.0),
      Vector4::new(0.0, 0.0, 0.0, -1.0)
    );
  
  let reflectivity = {
  let mut temp = 1.0;
    if draw.get_texture() == "terrain" {
      temp = 0.0;
    }
    temp
  };
  
  let lighting_colour: Matrix4<f32> =
    // (R, G, B, light type)
    Matrix4::from_cols(
      Vector4::new(0.0, 0.0, 1.0, point_light), // blue light
      Vector4::new(1.0, 0.0, 0.0, point_light),  // red light
      Vector4::new(0.4, 0.4, 0.4, directional_light), //sun
      Vector4::new(0.0, 0.0, 0.0, -1.0)
    );
  
  // (Intensity, 1)
  let attenuation: Matrix4<f32> =
    Matrix4::from_cols(
      Vector4::new(1.0, 0.25, 0.25, -1.0),
      Vector4::new(1.0, 0.25, 0.25, -1.0),
      Vector4::new(1.0, 0.0, 0.0, -1.0),
      Vector4::new(0.0, 0.0, 0.0, -1.0)
    );
  
  let uniform_data = vs_3d::ty::Data {
    transformation: transformation.into(),
    view : (view_matrix /* self.vk3d.scale*/).into(),
    proj : projection.into(),
    lightpositions: lighting_position.into(),
    lightcolours: lighting_colour.into(),
    attenuations: attenuation.into(),
  };
  
  uniform_buffer.next(uniform_data).unwrap()
}
