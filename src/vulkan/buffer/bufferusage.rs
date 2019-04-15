
#[derive(Clone)]
pub struct BufferUsage {
  transfer_src: bool,
  transfer_dst: bool,
  uniform_texel: bool,
  storage_texel: bool,
  uniform_buffer: bool,
  storage_buffer: bool,
  index_buffer: bool,
  vertex_buffer: bool,
  indirect_buffer: bool,
}

impl BufferUsage {
  pub fn none() -> BufferUsage {
    BufferUsage {
      transfer_src: false,
      transfer_dst: false,
      uniform_texel: false,
      storage_texel: false,
      uniform_buffer: false,
      storage_buffer: false,
      index_buffer: false,
      vertex_buffer: false,
      indirect_buffer: false,
    }
  }
  
  pub fn to_bits(&self) -> vk::BufferUsageFlags {
    let mut flags = 0;
    
    if self.transfer_src {
      flags = flags | vk::BUFFER_USAGE_TRANSFER_SRC_BIT;
    }
    if self.transfer_dst {
      flags = flags | vk::BUFFER_USAGE_TRANSFER_DST_BIT;
    }
    if self.uniform_texel {
      flags = flags | vk::BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT;
    }
    if self.storage_texel {
      flags = flags | vk::BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT;
    }
    if self.uniform_buffer {
      flags = flags | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT;
    }
    if self.storage_buffer {
      flags = flags | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT;
    }
    if self.index_buffer {
      flags = flags | vk::BUFFER_USAGE_INDEX_BUFFER_BIT;
    }
    if self.vertex_buffer {
      flags = flags | vk::BUFFER_USAGE_VERTEX_BUFFER_BIT;
    }
    if self.indirect_buffer {
      flags = flags | vk::BUFFER_USAGE_INDIRECT_BUFFER_BIT;
    }
    
    flags
  }
  
  pub fn set_as_transfer_dst(&mut self) {
    self.transfer_dst = true;
  }
  
  pub fn transfer_src_buffer() -> BufferUsage {
    BufferUsage {
      transfer_src: true,
      vertex_buffer: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn transfer_dst_buffer() -> BufferUsage {
    BufferUsage {
      transfer_dst: true,
      vertex_buffer: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn vertex_buffer() -> BufferUsage {
    BufferUsage {
      vertex_buffer: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn vertex_transfer_src_buffer() -> BufferUsage {
    BufferUsage {
      vertex_buffer: true,
      transfer_src: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn vertex_transfer_dst_buffer() -> BufferUsage {
    BufferUsage {
      vertex_buffer: true,
      transfer_dst: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn index_buffer() -> BufferUsage {
    BufferUsage {
      index_buffer: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn index_transfer_src_buffer() -> BufferUsage {
    BufferUsage {
      index_buffer: true,
      transfer_src: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn index_transfer_dst_buffer() -> BufferUsage {
    BufferUsage {
      index_buffer: true,
      transfer_dst: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn uniform_buffer() -> BufferUsage {
    BufferUsage {
      uniform_buffer: true,
      .. BufferUsage::none()
    }
  }
  
  pub fn storage_buffer() -> BufferUsage {
    BufferUsage {
      storage_buffer: true,
      .. BufferUsage::none()
    }
  }
}
