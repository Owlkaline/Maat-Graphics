use ash::vk;

use crate::vkwrapper::{Buffer, DescriptorSet, Image, Sampler, VkDevice};

pub struct DescriptorWriter {}

impl DescriptorWriter {
  pub fn builder() -> DescriptorWriterBuilder {
    DescriptorWriterBuilder::new()
  }
}

pub struct DescriptorWriterBuilder {
  descriptor_buffer_infos: Vec<vk::DescriptorBufferInfo>,
  descriptor_image_infos: Vec<vk::DescriptorImageInfo>,
  descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
}

impl DescriptorWriterBuilder {
  pub fn new() -> DescriptorWriterBuilder {
    DescriptorWriterBuilder {
      descriptor_buffer_infos: Vec::new(),
      descriptor_image_infos: Vec::new(),
      descriptor_write_sets: Vec::new(),
    }
  }

  pub fn update_buffer<T: Copy>(
    mut self,
    buffer: &Buffer<T>,
    descriptor_sets: &DescriptorSet,
  ) -> DescriptorWriterBuilder {
    self.descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
      buffer: *buffer.internal(),
      offset: 0,
      range: std::mem::size_of::<T>() as u64 * (buffer.data().len() as u64),
    });

    self.descriptor_write_sets.push(vk::WriteDescriptorSet {
      dst_set: descriptor_sets.internal()[0],
      dst_binding: self.descriptor_write_sets.len() as u32,
      descriptor_count: 1,
      descriptor_type: buffer.descriptor_usage(),
      p_buffer_info: &self.descriptor_buffer_infos[self.descriptor_buffer_infos.len() - 1],
      ..Default::default()
    });

    self
  }

  //pub fn update_storage_buffer<T: Copy>(
  //  mut self,
  //  storage_buffer: &Buffer<T>,
  //  descriptor_sets: &DescriptorSet,
  //) -> DescriptorWriterBuilder {
  //  self.descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
  //    buffer: *storage_buffer.internal(),
  //    offset: 0,
  //    range: std::mem::size_of::<T>() as u64 * (storage_buffer.data().len() as u64),
  //  });

  //  self.descriptor_write_sets.push(vk::WriteDescriptorSet {
  //    dst_set: descriptor_sets.internal()[0],
  //    dst_binding: self.descriptor_write_sets.len() as u32,
  //    descriptor_count: 1,
  //    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
  //    p_buffer_info: &self.descriptor_buffer_infos[self.descriptor_buffer_infos.len() - 1],
  //    ..Default::default()
  //  });

  //  self
  //}

  //pub fn update_uniform_buffer<T: Copy>(
  //  mut self,
  //  uniform_buffer: &Buffer<T>,
  //  descriptor_sets: &DescriptorSet,
  //) -> DescriptorWriterBuilder {
  //  self.descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
  //    buffer: *uniform_buffer.internal(),
  //    offset: 0,
  //    range: std::mem::size_of::<T>() as u64 * (uniform_buffer.data().len() as u64),
  //  });

  //  self.descriptor_write_sets.push(vk::WriteDescriptorSet {
  //    dst_set: descriptor_sets.internal()[0],
  //    dst_binding: self.descriptor_write_sets.len() as u32,
  //    descriptor_count: 1,
  //    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
  //    p_buffer_info: &self.descriptor_buffer_infos[self.descriptor_buffer_infos.len() - 1],
  //    ..Default::default()
  //  });

  //  self
  //}

  pub fn update_images(
    mut self,
    image: &Vec<Image>,
    sampler: &Vec<Sampler>,
    descriptor_sets: &DescriptorSet,
  ) -> DescriptorWriterBuilder {
    //self.descriptor_image_infos.push(vk::DescriptorImageInfo {
    //  image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    //  image_view: image.view(),
    //  sampler: sampler.internal(),
    //});
    let start_idx = self.descriptor_image_infos.len();
    for i in 0..image.len() {
      self.descriptor_image_infos.push(vk::DescriptorImageInfo {
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        image_view: image[i].view(),
        sampler: sampler[i].internal(),
      })
    }

    self.descriptor_write_sets.push(vk::WriteDescriptorSet {
      dst_set: descriptor_sets.internal()[0],
      dst_binding: start_idx as u32 + 1, //self.descriptor_write_sets.len() as u32,
      descriptor_count: 1,
      descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      p_image_info: self.descriptor_image_infos[start_idx..].as_ptr(), //[self.descriptor_image_infos.len() - 1],
      ..Default::default()
    });

    self
  }

  pub fn update_image(
    mut self,
    image: &Image,
    sampler: &Sampler,
    descriptor_sets: &DescriptorSet,
  ) -> DescriptorWriterBuilder {
    self.descriptor_image_infos.push(vk::DescriptorImageInfo {
      image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      image_view: image.view(),
      sampler: sampler.internal(),
    });

    self.descriptor_write_sets.push(vk::WriteDescriptorSet {
      dst_set: descriptor_sets.internal()[0],
      dst_binding: self.descriptor_write_sets.len() as u32,
      descriptor_count: 1,
      descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      p_image_info: &self.descriptor_image_infos[self.descriptor_image_infos.len() - 1],
      ..Default::default()
    });

    self
  }

  pub fn build(&self, device: &VkDevice) {
    unsafe {
      device
        .internal()
        .update_descriptor_sets(&self.descriptor_write_sets[..], &[]);
    }
  }
}
