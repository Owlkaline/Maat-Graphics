use vk;

pub enum AttachmentLoadOp {
  Load,
  Clear,
  DontCare,
}

pub enum AttachmentStoreOp {
  Store,
  DontCare,
}

pub enum ImageLayout {
  Undefined,
  General,
  ColourAttachmentOptimal,
  DepthStencilAttachmentOptimal,
  DepthStencilReadOnlyOptimal,
  ShaderReadOnlyOptimal,
  TransferSrcOptimal,
  TransferDstOptimal,
  PreInitialized,
  PresentSrcKHR,
}

pub enum Sample {
  Count1Bit,
  Count2Bit,
  Count4Bit,
  Count8Bit,
  Count16Bit,
}

pub enum FrontFace {
  CounterClockwise,
  Clockwise,
}

pub enum CullMode {
  None,
  Front,
  Back,
  FrontAndBack,
}

pub enum PolygonMode {
  Fill,
  Line,
  Point,
}

pub enum Topology {
  PointList,
  LineList,
  LineStrip,
  TriangleList,
  TriangleStrip,
  TriangleFan,
  LineListWithAdjacency,
  LineStripWithAdjacency,
  TriangleListWithAdjacency,
  TriangleStripWithAjacency,
  PatchList,
}

impl AttachmentLoadOp {
  pub fn to_bits(&self) -> vk::AttachmentLoadOp {
    match self {
      AttachmentLoadOp::Load => {
        vk::ATTACHMENT_LOAD_OP_LOAD
      },
      AttachmentLoadOp::Clear => {
        vk::ATTACHMENT_LOAD_OP_CLEAR
      },
      AttachmentLoadOp::DontCare => {
        vk::ATTACHMENT_LOAD_OP_DONT_CARE
      }
    }
  }
}

impl AttachmentStoreOp {
  pub fn to_bits(&self) -> vk::AttachmentStoreOp {
    match self {
      AttachmentStoreOp::Store => {
        vk::ATTACHMENT_STORE_OP_STORE
      },
      AttachmentStoreOp::DontCare => {
        vk::ATTACHMENT_STORE_OP_DONT_CARE
      },
    }
  }
}

impl ImageLayout {
  pub fn to_bits(&self) -> vk::ImageLayout {
    match self {
      ImageLayout::Undefined => {
        vk::IMAGE_LAYOUT_UNDEFINED
      },
      ImageLayout::General => {
        vk::IMAGE_LAYOUT_GENERAL
      },
      ImageLayout::ColourAttachmentOptimal => {
        vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL
      },
      ImageLayout::DepthStencilAttachmentOptimal => {
        vk::IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL
      },
      ImageLayout::DepthStencilReadOnlyOptimal => {
        vk::IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL
      },
      ImageLayout::ShaderReadOnlyOptimal => {
        vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
      },
      ImageLayout::TransferSrcOptimal => {
        vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL
      },
      ImageLayout::TransferDstOptimal => {
        vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
      },
      ImageLayout::PreInitialized => {
        vk::IMAGE_LAYOUT_PREINITIALIZED
      },
      ImageLayout::PresentSrcKHR => {
        vk::IMAGE_LAYOUT_PRESENT_SRC_KHR
      },
    }
  }
  
  pub fn to_attachment_reference(&self, index: u32) -> vk::AttachmentReference {
    match self {
      ImageLayout::Undefined => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_UNDEFINED
        }
      },
      ImageLayout::General => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_GENERAL
        }
      },
      ImageLayout::ColourAttachmentOptimal => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL
        }
      },
      ImageLayout::DepthStencilAttachmentOptimal => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        }
      },
      ImageLayout::DepthStencilReadOnlyOptimal => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL
        }
      },
      ImageLayout::ShaderReadOnlyOptimal => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
        }
      },
      ImageLayout::TransferSrcOptimal => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL
        }
      },
      ImageLayout::TransferDstOptimal => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
        }
      },
      ImageLayout::PreInitialized => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_PREINITIALIZED
        }
      },
      ImageLayout::PresentSrcKHR => {
        vk::AttachmentReference {
          attachment: index,
          layout: vk::IMAGE_LAYOUT_PRESENT_SRC_KHR
        }
      },
    }
  }
}

impl Sample {
  pub fn to_bits(&self) -> vk::SampleCountFlagBits {
    match self {
      Sample::Count1Bit => {
        vk::SAMPLE_COUNT_1_BIT
      },
      Sample::Count2Bit => {
        vk::SAMPLE_COUNT_2_BIT
      },
      Sample::Count4Bit => {
        vk::SAMPLE_COUNT_4_BIT
      },
      Sample::Count8Bit => {
        vk::SAMPLE_COUNT_8_BIT
      },
      Sample::Count16Bit => {
        vk::SAMPLE_COUNT_16_BIT
      }
    }
  }
}

impl FrontFace {
  pub fn to_bits(&self) -> vk::FrontFace {
    match self {
      FrontFace::CounterClockwise => {
        vk::FRONT_FACE_COUNTER_CLOCKWISE
      },
      FrontFace::Clockwise => {
        vk::FRONT_FACE_CLOCKWISE
      }
    }
  }
}

impl CullMode {
  pub fn to_bits(&self) -> vk::CullModeFlagBits {
    match self {
      CullMode::None => {
        vk::CULL_MODE_NONE
      },
      CullMode::Front => {
        vk::CULL_MODE_FRONT_BIT
      },
      CullMode::Back => {
        vk::CULL_MODE_BACK_BIT
      },
      CullMode::FrontAndBack => {
        vk::CULL_MODE_FRONT_AND_BACK
      }
    }
  }
}

impl PolygonMode {
  pub fn to_bits(&self) -> vk::PolygonMode {
    match self {
      PolygonMode::Fill => {
        vk::POLYGON_MODE_FILL
      },
      PolygonMode::Line => {
        vk::POLYGON_MODE_LINE
      },
      PolygonMode::Point => {
        vk::POLYGON_MODE_POINT
      }
    }
  }
}

impl Topology {
  pub fn to_bits(&self) -> vk::PrimitiveTopology {
    match self {
      Topology::PointList => {
        vk::PRIMITIVE_TOPOLOGY_POINT_LIST
      },
      Topology::LineList => {
        vk::PRIMITIVE_TOPOLOGY_LINE_LIST
      },
      Topology::LineStrip => {
        vk::PRIMITIVE_TOPOLOGY_LINE_STRIP
      },
      Topology::TriangleList => {
        vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST
      },
      Topology::TriangleStrip => {
        vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP
      },
      Topology::TriangleFan => {
        vk::PRIMITIVE_TOPOLOGY_TRIANGLE_FAN
      },
      Topology::LineListWithAdjacency => {
        vk::PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY
      },
      Topology::LineStripWithAdjacency => {
        vk::PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY
      },
      Topology::TriangleListWithAdjacency => {
        vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY
      },
      Topology::TriangleStripWithAjacency => {
        vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY
      },
      Topology::PatchList => {
        vk::PRIMITIVE_TOPOLOGY_PATCH_LIST
      }
    }
  }
}
