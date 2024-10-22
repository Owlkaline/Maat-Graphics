use ash::vk;

use crate::vkwrapper::VkDevice;

#[derive(Clone)]
pub struct DescriptorSet {
  descriptor_sets: Vec<vk::DescriptorSet>,
  descriptor_layouts: Vec<vk::DescriptorSetLayout>,
}

impl DescriptorSet {
  pub fn new(
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_layouts: Vec<vk::DescriptorSetLayout>,
  ) -> DescriptorSet {
    DescriptorSet {
      descriptor_sets,
      descriptor_layouts,
    }
  }

  pub fn builder() -> DescriptorSetBuilder {
    DescriptorSetBuilder::new()
  }

  pub fn internal(&self) -> &Vec<vk::DescriptorSet> {
    &self.descriptor_sets
  }

  pub fn layouts(&self) -> &Vec<vk::DescriptorSetLayout> {
    &self.descriptor_layouts
  }

  pub fn destroy(&self, device: &VkDevice) {
    unsafe {
      for layout in &self.descriptor_layouts {
        device
          .internal()
          .destroy_descriptor_set_layout(*layout, None);
      }
    }
  }
}

pub struct DescriptorSetBuilder {
  types: Vec<vk::DescriptorType>,
  stages: Vec<vk::ShaderStageFlags>,
}

impl DescriptorSetBuilder {
  pub fn new() -> DescriptorSetBuilder {
    let types: Vec<vk::DescriptorType> = Vec::new();
    let stages: Vec<vk::ShaderStageFlags> = Vec::new();

    DescriptorSetBuilder { types, stages }
  }

  pub fn uniform_buffer_vertex(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::UNIFORM_BUFFER);
    self.stages.push(vk::ShaderStageFlags::VERTEX);
    self
  }

  pub fn uniform_buffer_fragment(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::UNIFORM_BUFFER);
    self.stages.push(vk::ShaderStageFlags::FRAGMENT);
    self
  }

  pub fn combined_image_sampler_fragment(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
    self.stages.push(vk::ShaderStageFlags::FRAGMENT);
    self
  }

  pub fn combined_image_sampler_compute_fragment(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
    self
      .stages
      .push(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE);
    self
  }

  pub fn storage_vertex(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::STORAGE_BUFFER);
    self.stages.push(vk::ShaderStageFlags::VERTEX);
    self
  }

  pub fn storage_fragment(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::STORAGE_BUFFER);
    self.stages.push(vk::ShaderStageFlags::FRAGMENT);
    self
  }

  pub fn storage_compute(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::STORAGE_BUFFER);
    self.stages.push(vk::ShaderStageFlags::COMPUTE);
    self
  }

  pub fn storage_compute_fragment(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::STORAGE_BUFFER);
    self
      .stages
      .push(vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::FRAGMENT);
    self
  }

  pub fn storage_vertex_compute_fragment(mut self) -> DescriptorSetBuilder {
    self.types.push(vk::DescriptorType::STORAGE_BUFFER);
    self.stages.push(
      vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
    );
    self
  }

  pub fn build(&self, device: &VkDevice, descriptor_pool: &vk::DescriptorPool) -> DescriptorSet {
    let mut descriptor_layout_bindings = Vec::new();

    for i in 0..self.types.len() {
      descriptor_layout_bindings.push(
        *vk::DescriptorSetLayoutBinding::builder()
          .binding(i as u32)
          .descriptor_type(self.types[i])
          .descriptor_count(1)
          .stage_flags(self.stages[i]),
      );
    }

    let descriptor_info =
      vk::DescriptorSetLayoutCreateInfo::builder().bindings(&descriptor_layout_bindings);

    let descriptor_set_layouts = [unsafe {
      device
        .internal()
        .create_descriptor_set_layout(&descriptor_info, None)
        .unwrap()
    }];

    let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
      .descriptor_pool(*descriptor_pool)
      .set_layouts(&descriptor_set_layouts);

    let descriptor_sets = unsafe {
      device
        .internal()
        .allocate_descriptor_sets(&desc_alloc_info)
        .unwrap()
    };

    DescriptorSet::new(descriptor_sets, descriptor_set_layouts.to_vec())
  }
}
