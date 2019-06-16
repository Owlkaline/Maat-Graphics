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

#[derive(Clone, Copy, PartialEq)]
pub enum SampleCount {
  OneBit,
  TwoBit,
  FourBit,
  EightBit,
  SixteenBit,
  ThirtyTwoBit,
  SixtyFourBit,
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

#[derive(Clone)]
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

pub enum ShaderStage {
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

pub enum Access {
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
  
  // Daul
  ColourAttachmentReadAndWrite
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
  ComputeShader,
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

pub enum VertexInputRate {
  Vertex,
  Instance,
}

pub enum PresentMode {
  Mailbox,
  Fifo,
  Immediate,
}

pub enum CompositeAlpha {
  Opaque,
  PreMultiplied,
  PostMultiplied,
  Inherit,
}

pub enum ComponentSwizzle {
  Identity,
  Zero,
  One,
  R,
  G,
  B,
  A,
}

pub enum PhysicalDeviceType {
  Other,
  IntegratedGpu,
  DiscreteGpu,
  VirtualGpu,
  Cpu,
}

pub enum PipelineBindPoint {
  Graphics,
  Compute,
}

pub enum Dependency {
  ByRegion,
}

pub enum CommandBufferLevel {
  Primary,
  Secondary,
}

pub enum SubpassContents {
  Inline,
  SecondaryCommandBuffers,
}

pub enum IndexType {
  Uint16,
  Uint32,
}

pub enum CommandBufferUsage {
  OneTimeSubmit,
  RenderPassContinue,
  SimultaneousUse,
}

pub enum StencilOp {
  Keep,
  Zero,
  Replace,
  IncrementAndClamp,
  DecrementAndClamp,
  Invert,
  IncrementAndWrap,
  DecrementAndWrap,
}

pub enum ColourComponent {
  R,
  G,
  B,
  A,
}

pub enum LogicOp {
  Clear,
  And,
  AndReverse,
  Copy,
  AndInverted,
  NoOp,
  Xor,
  Or,
  Nor,
  Equivalent,
  Invert,
  OrReverse,
  CopyInverted,
  OrInverted,
  Nand,
  Set,
}

pub enum BlendOp {
  Add,
  Subtract,
  ReverseSubtract,
  Min,
  Max
}

pub enum DynamicState {
  Viewport,
  Scissor,
  LineWidth,
  DepthBias,
  BlendConstants,
  DepthBounds,
  StencilCompareMask,
  StencilWriteMask,
  StencilReference,
}

pub enum MemoryProperty {
  DeviceLocal,
  HostVisible,
  HostCoherent,
  HostCached,
  LazilyAllocated,
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

impl SampleCount {
  pub fn from(value: u32) -> SampleCount {
    let msaa;
    
    if value >= 64 {
      msaa = SampleCount::SixtyFourBit;
    } else if value >= 32 {
      msaa = SampleCount::ThirtyTwoBit;
    } else if value >= 16 {
      msaa = SampleCount::SixteenBit;
    } else if value >= 8 {
      msaa = SampleCount::EightBit;
    } else if value >= 4 {
      msaa = SampleCount::FourBit;
    } else if value >= 2 {
      msaa = SampleCount::TwoBit;
    } else {
      msaa = SampleCount::OneBit;
    }
    
    msaa
  }
  
  pub fn to_bits(&self) -> vk::SampleCountFlagBits {
    match self {
      SampleCount::OneBit => {
        vk::SAMPLE_COUNT_1_BIT
      },
      SampleCount::TwoBit => {
        vk::SAMPLE_COUNT_2_BIT
      },
      SampleCount::FourBit => {
        vk::SAMPLE_COUNT_4_BIT
      },
      SampleCount::EightBit => {
        vk::SAMPLE_COUNT_8_BIT
      },
      SampleCount::SixteenBit => {
        vk::SAMPLE_COUNT_16_BIT
      },
      SampleCount::ThirtyTwoBit => {
        vk::SAMPLE_COUNT_32_BIT
      },
      SampleCount::SixtyFourBit =>{
        vk::SAMPLE_COUNT_64_BIT
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
  
  pub fn depth_stencil_attachment() -> ImageUsage {
    ImageUsage {
      depth_stencil_attachment: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn depth_stencil_input_attachment() -> ImageUsage {
    ImageUsage {
      input_attachment: true,
      .. ImageUsage::depth_stencil_attachment()
    }
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
  
  pub fn transfer_src_dst_sampled() -> ImageUsage {
    ImageUsage {
      transfer_src: true,
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
  
  pub fn colour_attachment_sampled() -> ImageUsage {
    ImageUsage {
      sampled: true,
      colour_attachment: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn colour_input_attachment_sampled() -> ImageUsage {
    ImageUsage {
      input_attachment: true,
      .. ImageUsage::colour_attachment_sampled()
    }
  }
  
  pub fn colour_attachment_storage_sampled() -> ImageUsage {
    ImageUsage {
      sampled: true,
      .. ImageUsage::colour_attachment_sampled()
    }
  }
  
  pub fn transfer_src_colour_attachment_sampled() -> ImageUsage {
    ImageUsage {
      sampled: true,
      colour_attachment: true,
      transfer_src: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_src_colour_input_attachment_sampled() -> ImageUsage {
    ImageUsage {
      sampled: true,
      colour_attachment: true,
      transfer_src: true,
      input_attachment: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transfer_src_colour_attachment_sampled_storage() -> ImageUsage {
    ImageUsage {
      storage: true,
      .. ImageUsage::transfer_src_colour_attachment_sampled()
    }
  }
  
  pub fn colour_input_attachment_storage_sampled() -> ImageUsage {
    ImageUsage {
      sampled: true,
      storage: true,
      colour_attachment: true,
      input_attachment: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transient_colour_attachment() -> ImageUsage {
    ImageUsage {
      transient_attachment: true,
      colour_attachment: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transient_colour_input_attachment() -> ImageUsage {
    ImageUsage {
      transient_attachment: true,
      input_attachment: true,
      colour_attachment: true,
      .. ImageUsage::none()
    }
  }
  
  pub fn transient_depth_stencil_attachment() -> ImageUsage {
    ImageUsage {
      transient_attachment: true,
      depth_stencil_attachment: true,
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

impl ShaderStage {
  pub fn to_bits(&self) -> vk::ShaderStageFlagBits {
    match self {
      ShaderStage::Vertex => {
        vk::SHADER_STAGE_VERTEX_BIT
      },
      ShaderStage::TessellationControl => {
        vk::SHADER_STAGE_TESSELLATION_CONTROL_BIT
      },
      ShaderStage::Geometry => {
        vk::SHADER_STAGE_GEOMETRY_BIT
      },
      ShaderStage::Fragment => {
        vk::SHADER_STAGE_FRAGMENT_BIT
      },
      ShaderStage::Compute => {
        vk::SHADER_STAGE_COMPUTE_BIT
      },
      ShaderStage::AllGraphics => {
        vk::SHADER_STAGE_ALL_GRAPHICS
      },
      ShaderStage::All => {
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

impl Access {
  pub fn to_bits(&self) -> vk::AccessFlagBits {
    match self {
      Access::IndirectCommandRead => {
        vk::ACCESS_INDIRECT_COMMAND_READ_BIT
      },
      Access::IndexRead => {
        vk::ACCESS_INDEX_READ_BIT
      },
      Access::VertexAttributeRead => {
        vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT
      },
      Access::UniformRead => {
        vk::ACCESS_UNIFORM_READ_BIT
      },
      Access::InputAttachmentRead => {
        vk::ACCESS_INPUT_ATTACHMENT_READ_BIT
      },
      Access::ShaderRead => {
        vk::ACCESS_SHADER_READ_BIT
      },
      Access::ShaderWrite => {
        vk::ACCESS_SHADER_WRITE_BIT
      },
      Access::ColourAttachmentRead => {
        vk::ACCESS_COLOR_ATTACHMENT_READ_BIT
      },
      Access::ColourAttachmentWrite => {
        vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT
      },
      Access::DepthStencilAttachmentRead => {
        vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT
      },
      Access::DepthStencilAttachmentWrite => {
        vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT
      },
      Access::TransferRead => {
        vk::ACCESS_TRANSFER_READ_BIT
      },
      Access::TransferWrite => {
        vk::ACCESS_TRANSFER_WRITE_BIT
      },
      Access::HostRead => {
        vk::ACCESS_HOST_READ_BIT
      },
      Access::HostWrite => {
        vk::ACCESS_HOST_WRITE_BIT
      },
      Access::MemoryRead => {
        vk::ACCESS_MEMORY_READ_BIT
      },
      Access::MemoryWrite => {
        vk::ACCESS_MEMORY_WRITE_BIT
      },
      Access::ColourAttachmentReadAndWrite => {
        Access::ColourAttachmentRead.to_bits() | Access::ColourAttachmentWrite.to_bits()
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
      PipelineStage::ComputeShader => {
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

impl VertexInputRate {
  pub fn to_bits(&self) -> vk::VertexInputRate {
    match self {
      VertexInputRate::Vertex => {
        vk::VERTEX_INPUT_RATE_VERTEX
      },
      VertexInputRate::Instance => {
        vk::VERTEX_INPUT_RATE_INSTANCE
      },
    }
  }
}

impl PresentMode {
  pub fn to_bits(&self) -> vk::PresentModeKHR {
    match self {
      PresentMode::Mailbox => {
        vk::PRESENT_MODE_MAILBOX_KHR
      },
      PresentMode::Fifo => {
        vk::PRESENT_MODE_FIFO_KHR
      },
      PresentMode::Immediate => {
        vk::PRESENT_MODE_IMMEDIATE_KHR
      },
    }
  }
}

impl CompositeAlpha {
  pub fn to_bits(&self) -> vk::CompositeAlphaFlagBitsKHR {
    match self {
      CompositeAlpha::Opaque => {
        vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR
      },
      CompositeAlpha::PreMultiplied => {
        vk::COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR
      },
      CompositeAlpha::PostMultiplied => {
        vk::COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR
      },
      CompositeAlpha::Inherit => {
        vk::COMPOSITE_ALPHA_INHERIT_BIT_KHR
      },
    }
  }
}

impl ComponentSwizzle {
  pub fn to_bits(&self) -> vk::ComponentSwizzle {
    match self {
      ComponentSwizzle::Identity => {
        vk::COMPONENT_SWIZZLE_IDENTITY
      },
      ComponentSwizzle::Zero => {
        vk::COMPONENT_SWIZZLE_ZERO
      },
      ComponentSwizzle::One => {
        vk::COMPONENT_SWIZZLE_ONE
      },
      ComponentSwizzle::R => {
        vk::COMPONENT_SWIZZLE_R
      },
      ComponentSwizzle::G => {
        vk::COMPONENT_SWIZZLE_G
      },
      ComponentSwizzle::B => {
        vk::COMPONENT_SWIZZLE_B
      },
      ComponentSwizzle::A => {
        vk::COMPONENT_SWIZZLE_A
      },
    }
  }
}

impl PhysicalDeviceType {
  pub fn to_bits(&self) -> vk::PhysicalDeviceType {
    match self {
      PhysicalDeviceType::Other => {
        vk::PHYSICAL_DEVICE_TYPE_OTHER
      },
      PhysicalDeviceType::IntegratedGpu => {
        vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU
      },
      PhysicalDeviceType::DiscreteGpu => {
        vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU
      },
      PhysicalDeviceType::VirtualGpu => {
        vk::PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU
      },
      PhysicalDeviceType::Cpu => {
        vk::PHYSICAL_DEVICE_TYPE_CPU
      },
    }
  }
}

impl PipelineBindPoint {
  pub fn to_bits(&self) -> vk::PipelineBindPoint {
    match self {
      PipelineBindPoint::Graphics => {
        vk::PIPELINE_BIND_POINT_GRAPHICS
      },
      PipelineBindPoint::Compute => {
        vk::PIPELINE_BIND_POINT_COMPUTE
      },
    }
  }
}

impl Dependency {
  pub fn to_bits(&self) -> vk::DependencyFlagBits {
    match self {
      Dependency::ByRegion => {
        vk::DEPENDENCY_BY_REGION_BIT
      },
    }
  }
}

impl CommandBufferLevel {
  pub fn to_bits(&self) -> vk::CommandBufferLevel {
    match self {
      CommandBufferLevel::Primary => {
        vk::COMMAND_BUFFER_LEVEL_PRIMARY
      },
      CommandBufferLevel::Secondary => {
        vk::COMMAND_BUFFER_LEVEL_SECONDARY
      },
    }
  }
}

impl SubpassContents {
  pub fn to_bits(&self) -> vk::SubpassContents {
    match self {
      SubpassContents::Inline => {
        vk::SUBPASS_CONTENTS_INLINE
      },
      SubpassContents::SecondaryCommandBuffers => {
        vk::SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS
      },
    }
  }
}

impl IndexType {
  pub fn to_bits(&self) -> vk::IndexType {
    match self {
      IndexType::Uint16 => {
        vk::INDEX_TYPE_UINT16
      },
      IndexType::Uint32 => {
        vk::INDEX_TYPE_UINT32
      },
    }
  }
}

impl CommandBufferUsage {
  pub fn to_bits(&self) -> vk::CommandBufferUsageFlagBits {
    match self {
      CommandBufferUsage::OneTimeSubmit => {
        vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT
      },
      CommandBufferUsage::RenderPassContinue => {
        vk::COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT
      },
      CommandBufferUsage::SimultaneousUse => {
        vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT
      },
    }
  }
}

impl StencilOp {
  pub fn to_bits(&self) -> vk::StencilOp {
    match self {
      StencilOp::Keep => {
        vk::STENCIL_OP_KEEP
      },
      StencilOp::Zero => {
        vk::STENCIL_OP_ZERO
      },
      StencilOp::Replace => {
        vk::STENCIL_OP_REPLACE
      },
      StencilOp::IncrementAndClamp => {
        vk::STENCIL_OP_INCREMENT_AND_CLAMP
      },
      StencilOp::DecrementAndClamp => {
        vk::STENCIL_OP_DECREMENT_AND_CLAMP
      },
      StencilOp::Invert => {
        vk::STENCIL_OP_INVERT
      },
      StencilOp::IncrementAndWrap => {
        vk::STENCIL_OP_INCREMENT_AND_WRAP
      },
      StencilOp::DecrementAndWrap => {
        vk::STENCIL_OP_DECREMENT_AND_WRAP
      },
    }
  }
}

impl ColourComponent {
  pub fn to_bits(&self) -> vk::ColorComponentFlagBits {
    match self {
      ColourComponent::R => {
        vk::COLOR_COMPONENT_R_BIT
      },
      ColourComponent::G => {
        vk::COLOR_COMPONENT_G_BIT
      },
      ColourComponent::B => {
        vk::COLOR_COMPONENT_B_BIT
      },
      ColourComponent::A => {
        vk::COLOR_COMPONENT_A_BIT
      },
    }
  }
}

impl LogicOp {
  pub fn to_bits(&self) -> vk::LogicOp {
    match self {
      LogicOp::Clear => {
        vk::LOGIC_OP_CLEAR
      },
      LogicOp::And => {
        vk::LOGIC_OP_AND
      },
      LogicOp::AndReverse => {
        vk::LOGIC_OP_AND_REVERSE
      },
      LogicOp::Copy => {
        vk::LOGIC_OP_COPY
      },
     LogicOp:: AndInverted => {
        vk::LOGIC_OP_AND_INVERTED
      },
      LogicOp::NoOp => {
        vk::LOGIC_OP_NO_OP
      },
      LogicOp::Xor => {
        vk::LOGIC_OP_XOR
      },
      LogicOp::Or => {
        vk::LOGIC_OP_OR
      },
      LogicOp::Nor => {
        vk::LOGIC_OP_NOR
      },
      LogicOp::Equivalent => {
        vk::LOGIC_OP_EQUIVALENT
      },
      LogicOp::Invert => {
        vk::LOGIC_OP_INVERT
      },
      LogicOp::OrReverse => {
        vk::LOGIC_OP_OR_REVERSE
      },
      LogicOp::CopyInverted => {
        vk::LOGIC_OP_COPY_INVERTED
      },
      LogicOp::OrInverted => {
        vk::LOGIC_OP_OR_INVERTED
      },
      LogicOp::Nand => {
        vk::LOGIC_OP_NAND
      },
      LogicOp::Set => {
        vk::LOGIC_OP_SET
      },
    }
  }
}

impl BlendOp {
  pub fn to_bits(&self) -> vk::BlendOp {
    match self {
      BlendOp::Add => {
        vk::BLEND_OP_ADD
      },
      BlendOp::Subtract => {
        vk::BLEND_OP_SUBTRACT
      },
      BlendOp::ReverseSubtract => {
        vk::BLEND_OP_REVERSE_SUBTRACT
      },
      BlendOp::Min => {
        vk::BLEND_OP_MIN
      },
      BlendOp::Max => {
        vk::BLEND_OP_MAX
      },
    }
  }
}

impl DynamicState {
  pub fn to_bits(&self) -> vk::DynamicState {
    match self {
      DynamicState::Viewport => {
        vk::DYNAMIC_STATE_VIEWPORT
      },
      DynamicState::Scissor => {
        vk::DYNAMIC_STATE_SCISSOR
      },
      DynamicState::LineWidth => {
        vk::DYNAMIC_STATE_LINE_WIDTH
      },
      DynamicState::DepthBias => {
        vk::DYNAMIC_STATE_DEPTH_BIAS
      },
      DynamicState::BlendConstants => {
        vk::DYNAMIC_STATE_BLEND_CONSTANTS
      },
      DynamicState::DepthBounds => {
        vk::DYNAMIC_STATE_DEPTH_BOUNDS
      },
      DynamicState::StencilCompareMask => {
        vk::DYNAMIC_STATE_STENCIL_COMPARE_MASK
      },
      DynamicState::StencilWriteMask => {
        vk::DYNAMIC_STATE_STENCIL_WRITE_MASK
      },
      DynamicState::StencilReference => {
        vk::DYNAMIC_STATE_STENCIL_REFERENCE
      },
    }
  }
}

impl MemoryProperty {
  pub fn to_bits(&self) -> vk::MemoryPropertyFlagBits {
    match self {
      MemoryProperty::DeviceLocal => {
        vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT
      },
      MemoryProperty::HostVisible => {
        vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT
      },
      MemoryProperty::HostCoherent => {
        vk::MEMORY_PROPERTY_HOST_COHERENT_BIT
      },
      MemoryProperty::HostCached => {
        vk::MEMORY_PROPERTY_HOST_CACHED_BIT
      },
      MemoryProperty::LazilyAllocated => {
        vk::MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT
      },
    }
  }
}



