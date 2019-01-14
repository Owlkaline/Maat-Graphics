use vk;

pub enum VkBool {
  True,
  False,
}

pub enum AttachmentLoadOp {
  Load,
  Clear,
  DontCare,
}

pub enum AttachmentStoreOp {
  Store,
  DontCare,
}

#[derive(Clone, PartialEq)]
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

pub enum Filter {
  Nearest,
  Linear,
}

pub enum MipmapMode {
  Nearest,
  Linear,
}

#[derive(Clone)]
pub enum AddressMode {
  Repeat,
  MirroredRepeat,
  ClampToEdge,
  ClampToBorder,
  MirrorClampToEdge,
}

pub enum BorderColour {
  FloatTransparentBlack,
  FloatOpaqueBlack,
  FloatOpaqueWhite,
  IntTransparentBlack,
  IntOpaqueBlack,
  IntOpaqueWhite,
}

pub enum CompareOp {
  Never,
  Less,
  Equal,
  LessOrEqual,
  Greater,
  NotEqual,
  GreaterOrEqual,
  Always,
}

pub enum ImageType {
  Type1D,
  Type2D,
  Type3D,
}

pub enum ImageTiling {
  Optimal,
  Linear,
}

pub struct ImageUsage {
  pub transfer_src: bool,
  pub transfer_dst: bool,
  pub sampled: bool,
  pub storage: bool,
  pub colour_attachment: bool,
  pub depth_stencil_attachment: bool,
  pub transient_attachment: bool,
  pub input_attachment: bool,
}

pub enum SharingMode {
  Exclusive,
  Concurrent,
}

pub enum ImageViewType {
  Type1D,
  Type2D,
  Type3D,
  TypeCube,
  Type1DArray,
  Type2DArray,
  TypeCubeArray,
}

pub enum ShaderStageFlagBits {
  Vertex,
  TessellationControl,
  Geometry,
  Fragment,
  Compute,
  AllGraphics,
  All,
}

pub enum DescriptorType {
  Sampler,
  CombinedImageSampler,
  SampledImage,
  StorageImage,
  UniformTexelBuffer,
  StorageTexelBuffer,
  UniformBuffer,
  StorageBuffer,
  UniformBufferDynamic,
  StorageBufferDynamic,
  InputAttachment,
}

pub enum AccessFlagBits {
  IndirectCommandRead,
  IndexRead,
  VertexAttributeRead,
  UniformRead,
  InputAttachmentRead,
  ShaderRead,
  ShaderWrite,
  ColourAttachmentRead,
  ColourAttachmentWrite,
  DepthStencilAttachmentRead,
  DepthStencilAttachmentWrite,
  TransferRead,
  TransferWrite,
  HostRead,
  HostWrite,
  MemoryRead,
  MemoryWrite,
}

pub enum ImageAspect {
  Colour,
  Depth,
  Stencil,
  MetaData,
}

pub enum PipelineStage {
  TopOfPipe,
  DrawIndirect,
  VertexInput,
  VertexShader,
  TessellationControlShader,
  TessellationEvaluationShader,
  GeometryShader,
  FragmentShader,
  EarlyFragmentTests,
  LateFragementTests,
  ColorAttachmentOutput,
  ComputShader,
  Transfer,
  BottomOfPipe,
  Host,
  AllGraphics,
  AllCommands,
}

pub enum BlendFactor {
  Zero,
  One,
  SrcColour,
  OneMinusSrcColour,
  DstColour,
  OneMinusDstColour,
  SrcAlpha,
  OneMinusSrcAlpha,
  DstAlpha,
  OneMinusDstAlpha,
  ConstantColour,
  OneMinusConstantColour,
  SrcAlphaSaturate,
}

impl VkBool {
  pub fn to_bits(&self) -> vk::Bool32 {
    match self {
      VkBool::True => {
        vk::TRUE
      },
      VkBool::False => {
        vk::FALSE
      }
    }
  }
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

impl Filter {
  pub fn to_bits(&self) -> vk::Filter {
    match self {
      Filter::Nearest => {
        vk::FILTER_NEAREST
      },
      Filter::Linear => {
        vk::FILTER_LINEAR
      },
    }
  }
}

impl MipmapMode {
  pub fn to_bits(&self) -> vk::SamplerMipmapMode {
    match self {
      MipmapMode::Nearest => {
        vk::SAMPLER_MIPMAP_MODE_NEAREST
      },
      MipmapMode::Linear => {
        vk::SAMPLER_MIPMAP_MODE_LINEAR
      },
    }
  }
}

impl AddressMode {
  pub fn to_bits(&self) -> vk::SamplerAddressMode {
    match self {
      AddressMode::Repeat => {
        vk::SAMPLER_ADDRESS_MODE_REPEAT
      },
      AddressMode::MirroredRepeat => {
        vk::SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT
      },
      AddressMode::ClampToEdge => {
        vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE
      },
      AddressMode::ClampToBorder => {
        vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER
      },
      AddressMode::MirrorClampToEdge => {
        vk::SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE
      },
    }
  }
}

impl BorderColour {
  pub fn to_bits(&self) -> vk::BorderColor {
    match self {
      BorderColour::FloatTransparentBlack => {
        vk::BORDER_COLOR_FLOAT_TRANSPARENT_BLACK
      },
      BorderColour::FloatOpaqueBlack => {
        vk::BORDER_COLOR_FLOAT_OPAQUE_BLACK
      },
      BorderColour::FloatOpaqueWhite => {
        vk::BORDER_COLOR_FLOAT_OPAQUE_WHITE
      },
      BorderColour::IntTransparentBlack => {
        vk::BORDER_COLOR_INT_TRANSPARENT_BLACK
      },
      BorderColour::IntOpaqueBlack => {
        vk::BORDER_COLOR_INT_OPAQUE_BLACK
      },
      BorderColour::IntOpaqueWhite => {
        vk::BORDER_COLOR_INT_OPAQUE_WHITE
      },
    }
  }
}

impl CompareOp {
  pub fn to_bits(&self) -> vk::CompareOp {
    match self {
      CompareOp::Never => {
        vk::COMPARE_OP_NEVER
      },
      CompareOp::Less => {
        vk::COMPARE_OP_LESS
      },
      CompareOp::Equal => {
        vk::COMPARE_OP_EQUAL
      },
      CompareOp::LessOrEqual => {
        vk::COMPARE_OP_LESS_OR_EQUAL
      },
      CompareOp::Greater => {
        vk::COMPARE_OP_GREATER
      },
      CompareOp::NotEqual => {
        vk::COMPARE_OP_NOT_EQUAL
      },
      CompareOp::GreaterOrEqual => {
        vk::COMPARE_OP_GREATER_OR_EQUAL
      },
      CompareOp::Always => {
        vk::COMPARE_OP_ALWAYS
      },
    }
  }
}

impl ImageType {
  pub fn to_bits(&self) -> vk::ImageType {
    match self {
      ImageType::Type1D => {
        vk::IMAGE_TYPE_1D
      },
      ImageType::Type2D => {
        vk::IMAGE_TYPE_2D
      },
      ImageType::Type3D => {
        vk::IMAGE_TYPE_3D
      },
    }
  }
}

impl ImageTiling {
  pub fn to_bits(&self) -> vk::ImageTiling {
    match self {
      ImageTiling::Optimal => {
        vk::IMAGE_TILING_OPTIMAL
      },
      ImageTiling::Linear => {
        vk::IMAGE_TILING_LINEAR
      },
    }
  }
}

impl ImageUsage {
  pub fn none() -> ImageUsage {
    ImageUsage {
      transfer_src: false,
      transfer_dst: false,
      sampled: false,
      storage: false,
      colour_attachment: false,
      depth_stencil_attachment: false,
      transient_attachment: false,
      input_attachment: false,
    }
  }
  
  pub fn to_bits(&self) -> vk::ImageUsageFlagBits {
    let mut flags = 0;
    
    if self.transfer_src {
      flags = flags | vk::IMAGE_USAGE_TRANSFER_SRC_BIT;
    }
    if self.transfer_dst {
      flags = flags | vk::IMAGE_USAGE_TRANSFER_DST_BIT;
    }
    if self.sampled {
      flags = flags | vk::IMAGE_USAGE_SAMPLED_BIT;
    }
    if self.storage {
      flags = flags | vk::IMAGE_USAGE_STORAGE_BIT;
    }
    if self.colour_attachment {
      flags = flags | vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    }
    if self.depth_stencil_attachment {
      flags = flags | vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
    }
    if self.transient_attachment {
      flags = flags | vk::IMAGE_USAGE_TRANSIENT_ATTACHMENT_BIT;
    }
    if self.input_attachment {
      flags = flags | vk::IMAGE_USAGE_INPUT_ATTACHMENT_BIT;
    }
    
    flags
  }
  
  pub fn transfer_src() -> ImageUsage {
    ImageUsage {
      transfer_src: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_dst() -> ImageUsage {
    ImageUsage {
      transfer_dst: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_src_sampled() -> ImageUsage {
    ImageUsage {
      transfer_src: true,
      sampled: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_dst_sampled() -> ImageUsage {
    ImageUsage {
      transfer_dst: true,
      sampled: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_src_storage() -> ImageUsage {
    ImageUsage {
      transfer_src: true,
      storage: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_dst_storage() -> ImageUsage {
    ImageUsage {
      transfer_dst: true,
      storage: true,
      .. ImageUsage::none()
    }
  }
}

impl SharingMode {
  pub fn to_bits(&self) -> vk::SharingMode {
    match self {
      SharingMode::Exclusive => {
        vk::SHARING_MODE_EXCLUSIVE
      },
      SharingMode::Concurrent => {
        vk::SHARING_MODE_CONCURRENT
      },
    }
  }
}

impl ImageViewType {
  pub fn to_bits(&self) -> vk::ImageViewType {
    match self {
      ImageViewType::Type1D => {
        vk::IMAGE_VIEW_TYPE_1D
      },
      ImageViewType::Type2D => {
        vk::IMAGE_VIEW_TYPE_2D
      },
      ImageViewType::Type3D => {
        vk::IMAGE_VIEW_TYPE_3D
      },
      ImageViewType::TypeCube => {
        vk::IMAGE_VIEW_TYPE_CUBE
      },
      ImageViewType::Type1DArray => {
        vk::IMAGE_VIEW_TYPE_1D_ARRAY
      },
      ImageViewType::Type2DArray => {
        vk::IMAGE_VIEW_TYPE_2D_ARRAY
      },
      ImageViewType::TypeCubeArray => {
        vk::IMAGE_VIEW_TYPE_CUBE_ARRAY
      },
    }
  }
}

impl ShaderStageFlagBits {
  pub fn to_bits(&self) -> vk::ShaderStageFlagBits {
    match self {
      ShaderStageFlagBits::Vertex => {
        vk::SHADER_STAGE_VERTEX_BIT
      },
      ShaderStageFlagBits::TessellationControl => {
        vk::SHADER_STAGE_TESSELLATION_CONTROL_BIT
      },
      ShaderStageFlagBits::Geometry => {
        vk::SHADER_STAGE_GEOMETRY_BIT
      },
      ShaderStageFlagBits::Fragment => {
        vk::SHADER_STAGE_FRAGMENT_BIT
      },
      ShaderStageFlagBits::Compute => {
        vk::SHADER_STAGE_COMPUTE_BIT
      },
      ShaderStageFlagBits::AllGraphics => {
        vk::SHADER_STAGE_ALL_GRAPHICS
      },
      ShaderStageFlagBits::All => {
        vk::SHADER_STAGE_ALL
      },
    }
  }
}

impl DescriptorType {
  pub fn to_bits(&self) -> vk::DescriptorType {
    match self {
      DescriptorType::Sampler => {
        vk::DESCRIPTOR_TYPE_SAMPLER
      },
      DescriptorType::CombinedImageSampler => {
        vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
      },
      DescriptorType::SampledImage => {
        vk::DESCRIPTOR_TYPE_SAMPLED_IMAGE
      },
      DescriptorType::StorageImage => {
        vk::DESCRIPTOR_TYPE_STORAGE_IMAGE
      },
      DescriptorType::UniformTexelBuffer => {
        vk::DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER
      },
      DescriptorType::StorageTexelBuffer => {
        vk::DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER
      },
      DescriptorType::UniformBuffer => {
        vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER
      },
      DescriptorType::StorageBuffer => {
        vk::DESCRIPTOR_TYPE_STORAGE_BUFFER
      },
      DescriptorType::UniformBufferDynamic => {
        vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
      },
      DescriptorType::StorageBufferDynamic => {
        vk::DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
      },
      DescriptorType::InputAttachment => {
        vk::DESCRIPTOR_TYPE_INPUT_ATTACHMENT
      },
    }
  }
}

impl AccessFlagBits {
  pub fn to_bits(&self) -> vk::AccessFlagBits {
    match self {
      AccessFlagBits::IndirectCommandRead => {
        vk::ACCESS_INDIRECT_COMMAND_READ_BIT
      },
      AccessFlagBits::IndexRead => {
        vk::ACCESS_INDEX_READ_BIT
      },
      AccessFlagBits::VertexAttributeRead => {
        vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT
      },
      AccessFlagBits::UniformRead => {
        vk::ACCESS_UNIFORM_READ_BIT
      },
      AccessFlagBits::InputAttachmentRead => {
        vk::ACCESS_INPUT_ATTACHMENT_READ_BIT
      },
      AccessFlagBits::ShaderRead => {
        vk::ACCESS_SHADER_READ_BIT
      },
      AccessFlagBits::ShaderWrite => {
        vk::ACCESS_SHADER_WRITE_BIT
      },
      AccessFlagBits::ColourAttachmentRead => {
        vk::ACCESS_COLOR_ATTACHMENT_READ_BIT
      },
      AccessFlagBits::ColourAttachmentWrite => {
        vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT
      },
      AccessFlagBits::DepthStencilAttachmentRead => {
        vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT
      },
      AccessFlagBits::DepthStencilAttachmentWrite => {
        vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT
      },
      AccessFlagBits::TransferRead => {
        vk::ACCESS_TRANSFER_READ_BIT
      },
      AccessFlagBits::TransferWrite => {
        vk::ACCESS_TRANSFER_WRITE_BIT
      },
      AccessFlagBits::HostRead => {
        vk::ACCESS_HOST_READ_BIT
      },
      AccessFlagBits::HostWrite => {
        vk::ACCESS_HOST_WRITE_BIT
      },
      AccessFlagBits::MemoryRead => {
        vk::ACCESS_MEMORY_READ_BIT
      },
      AccessFlagBits::MemoryWrite => {
        vk::ACCESS_MEMORY_WRITE_BIT
      },
    }
  }
}

impl ImageAspect {
  pub fn to_bits(&self) -> vk::ImageAspectFlagBits {
    match self {
      ImageAspect::Colour => {
        vk::IMAGE_ASPECT_COLOR_BIT
      },
      ImageAspect::Depth => {
        vk::IMAGE_ASPECT_DEPTH_BIT
      },
      ImageAspect::Stencil => {
        vk::IMAGE_ASPECT_STENCIL_BIT
      },
      ImageAspect::MetaData => {
        vk::IMAGE_ASPECT_METADATA_BIT
      },
    }
  }
}

impl PipelineStage {
  pub fn to_bits(&self) -> vk::PipelineStageFlagBits {
    match self {
      PipelineStage::TopOfPipe => {
        vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT
      },
      PipelineStage::DrawIndirect => {
        vk::PIPELINE_STAGE_DRAW_INDIRECT_BIT
      },
      PipelineStage::VertexInput => {
        vk::PIPELINE_STAGE_VERTEX_INPUT_BIT
      },
      PipelineStage::VertexShader => {
        vk::PIPELINE_STAGE_VERTEX_SHADER_BIT
      },
      PipelineStage::TessellationControlShader => {
        vk::PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT
      },
      PipelineStage::TessellationEvaluationShader => {
        vk::PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT
      },
      PipelineStage::GeometryShader => {
        vk::PIPELINE_STAGE_GEOMETRY_SHADER_BIT
      },
      PipelineStage::FragmentShader => {
        vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT
      },
      PipelineStage::EarlyFragmentTests => {
        vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT
      },
      PipelineStage::LateFragementTests => {
        vk::PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT
      },
      PipelineStage::ColorAttachmentOutput => {
        vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
      },
      PipelineStage::ComputShader => {
        vk::PIPELINE_STAGE_COMPUTE_SHADER_BIT
      },
      PipelineStage::Transfer => {
        vk::PIPELINE_STAGE_TRANSFER_BIT
      },
      PipelineStage::BottomOfPipe => {
        vk::PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT
      },
      PipelineStage::Host => {
        vk::PIPELINE_STAGE_HOST_BIT
      },
      PipelineStage::AllGraphics => {
        vk::PIPELINE_STAGE_ALL_GRAPHICS_BIT
      },
      PipelineStage::AllCommands => {
        vk::PIPELINE_STAGE_ALL_COMMANDS_BIT
      },
    }
  }
}

impl BlendFactor {
  pub fn to_bits(&self) -> vk::BlendFactor {
    match self {
      BlendFactor::Zero => {
        vk::BLEND_FACTOR_ZERO
      },
      BlendFactor::One => {
        vk::BLEND_FACTOR_ONE
      },
      BlendFactor::SrcColour => {
        vk::BLEND_FACTOR_SRC_COLOR
      },
      BlendFactor::OneMinusSrcColour => {
        vk::BLEND_FACTOR_ONE_MINUS_SRC_COLOR
      },
      BlendFactor::DstColour => {
        vk::BLEND_FACTOR_DST_COLOR
      },
      BlendFactor::OneMinusDstColour => {
        vk::BLEND_FACTOR_ONE_MINUS_DST_COLOR
      },
      BlendFactor::SrcAlpha => {
        vk::BLEND_FACTOR_SRC_ALPHA
      },
      BlendFactor::OneMinusSrcAlpha => {
        vk::BLEND_FACTOR_ONE_MINUS_SRC_ALPHA
      },
      BlendFactor::DstAlpha => {
        vk::BLEND_FACTOR_DST_ALPHA
      },
      BlendFactor::OneMinusDstAlpha => {
        vk::BLEND_FACTOR_ONE_MINUS_DST_ALPHA
      },
      BlendFactor::ConstantColour => {
        vk::BLEND_FACTOR_CONSTANT_COLOR
      },
      BlendFactor::OneMinusConstantColour => {
        vk::BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR
      },
      BlendFactor::SrcAlphaSaturate => {
        vk::BLEND_FACTOR_SRC_ALPHA_SATURATE
      },
    }
  }
}
