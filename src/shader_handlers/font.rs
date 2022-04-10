use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;

use ash::vk;
use image;

use crate::offset_of;
use std::io::Cursor;

use crate::glam::{Vec2, Vec4};
use crate::shader_handlers::TextureHandler;
use crate::vkwrapper::Image as vkImage;
use crate::vkwrapper::{
  Buffer, DescriptorPoolBuilder, DescriptorSet, DescriptorWriter, GraphicsPipelineBuilder, Sampler,
  Shader, VkDevice, Vulkan,
};

use std::collections::HashMap;

const LINE_HEIGHT: f32 = 0.03;
const DESIRED_PADDING: i32 = 3;
const SPACE_ASCII: i32 = 32;
const NEW_LINE: char = '\n';

const PAD_TOP: usize = 0;
const PAD_LEFT: usize = 1;
const PAD_BOTTOM: usize = 2;
const PAD_RIGHT: usize = 3;

#[derive(Clone, Debug, Copy)]
pub struct TextVertex {
  pub pos: [f32; 4],
  pub uv: [f32; 4],
  pub colour: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Character {
  id: i32,
  x_coord: f32,
  y_coord: f32,
  x_max_coord: f32,
  y_max_coord: f32,
  x_offset: f32,
  y_offset: f32,
  size_x: f32,
  size_y: f32,
  x_advance: f32,
}

pub struct Meta {
  character_data: HashMap<i32, Character>,

  values: HashMap<String, String>,
  buffer: BufReader<File>,
  padding: Vec<i32>,
  padding_width: i32,
  padding_height: i32,
  vert_per_pixel_size: f32,
  horz_per_pixel_size: f32,
  space_width: f32,
}

#[derive(PartialEq)]
pub struct TextMeshCreator {
  meta: Meta,
}

pub struct TextMeshData {
  data: Vec<TextVertex>,
}

#[derive(Clone, Debug)]
pub struct Word {
  width: f32,
  font_size: f32,
  characters: Vec<Character>,
}

#[derive(Debug)]
pub struct Line {
  max_length: f32,
  space_size: f32,
  words: Vec<Word>,
  current_line_length: f32,
}

pub struct FontType {
  texture: vkImage,
  pool: vk::DescriptorPool,
  loader: TextMeshCreator,
  descriptor: DescriptorSet,
  shader: Shader<TextVertex>,
}

pub struct GuiText {
  text: String,
  font_size: f32,

  descriptor_set: Option<DescriptorSet>,
  colour: Vec4,
  position: Vec2,
  max_line_size: f32,
  number_of_lines: i32,

  //font: FontType,
  center_text: bool,
}

pub struct TextMaster {
  descriptor_pool: vk::DescriptorPool,
  //texts: HashMap<FontType, Vec<GuiText>>,
  font_type: FontType,
  text: Vec<(GuiText, Buffer<TextVertex>)>,
  unused_text: Vec<((GuiText, Buffer<TextVertex>), u32)>,
}

impl TextMaster {
  pub fn new(vulkan: &mut Vulkan, font: FontType) -> TextMaster {
    let descriptor_pool = DescriptorPoolBuilder::new()
      .num_uniform_buffers(100)
      .build(vulkan.device());

    TextMaster {
      descriptor_pool,
      font_type: font,
      text: Vec::new(),
      unused_text: Vec::new(),
    }
  }

  pub fn load_text(&mut self, mut text: GuiText, vulkan: &mut Vulkan) {
    let mut already_exists = false;
    for i in 0..self.text.len() {
      if self.text[i].0.text() == text.text() && self.text[i].0.font_size() == text.font_size() {
        self.text[i].0.set_position(text.position());
        self.text[i].0.set_colour(text.colour());
        already_exists = true;
        break;
      }
    }
    for i in 0..self.unused_text.len() {
      if self.unused_text[i].0 .0.text() == text.text()
        && self.unused_text[i].0 .0.font_size() == text.font_size()
      {
        let mut gui_text = self.unused_text.remove(i).0;
        gui_text.0.set_position(text.position());
        gui_text.0.set_colour(text.colour());
        self.text.push(gui_text);
        already_exists = true;
        break;
      }
    }

    if !already_exists {
      let data = self.font_type.load_text(&mut text);

      self.text.push((
        text,
        Buffer::<TextVertex>::new_vertex(vulkan.device(), data.data().clone()),
      ));
    }
  }

  pub fn remove_text(&mut self, text: GuiText, device: &VkDevice) {
    let mut should_remove = None;
    for i in 0..self.text.len() {
      if self.text[i].0 == text {
        should_remove = Some(i);
      }
    }

    if let Some(i) = should_remove {
      self.text[i].1.destroy(device);
      self.text.remove(i);
    }
  }

  pub fn remove_all_text(&mut self, device: &VkDevice) {
    for i in (0..self.text.len()).rev() {
      self.text[i].1.destroy(device);
      self.text.remove(i);
    }
  }

  pub fn remove_unused_text(&mut self, text_this_draw: Vec<String>, device: &VkDevice) {
    for i in (0..self.unused_text.len()).rev() {
      self.unused_text[i].1 += 1;
      if self.unused_text[i].1 > 600 {
        self.unused_text[i].0 .1.destroy(device);
        self.unused_text.remove(i);
      }
    }

    for i in (0..self.text.len()).rev() {
      if !text_this_draw.contains(&self.text[i].0.text()) {
        //self.text[i].1.destroy(device);
        let gui_text = self.text.remove(i);
        self.unused_text.push((gui_text, 0));
      }
    }
  }

  pub fn text(&self) -> &Vec<(GuiText, Buffer<TextVertex>)> {
    &self.text
  }

  pub fn font(&self) -> &FontType {
    &self.font_type
  }
}

impl TextMeshCreator {
  pub fn new(file: String) -> TextMeshCreator {
    TextMeshCreator {
      meta: Meta::load_font_data(file),
    }
  }

  pub fn create_text_data(&mut self, text: &mut GuiText) -> TextMeshData {
    let lines = self.create_structure(&text);
    self.create_quad_verticies(text, lines)
  }

  fn create_structure(&mut self, text: &GuiText) -> Vec<Line> {
    let raw_text = text.text();
    let chars = raw_text.chars();
    let mut lines = Vec::new();

    let mut current_line = Line::new(
      self.meta.space_width(),
      text.font_size(),
      text.max_line_size(),
    );

    let mut current_word = Word::new(text.font_size());

    for c in chars {
      let ascii: i32 = c as i32;
      if ascii == SPACE_ASCII {
        let added = current_line.attempt_to_add_word(&current_word);
        if !added {
          lines.push(current_line);
          current_line = Line::new(
            self.meta.space_width(),
            text.font_size(),
            text.max_line_size(),
          );
          current_line.attempt_to_add_word(&current_word);
        }
        current_word = Word::new(text.font_size());
        continue;
      } else if c == NEW_LINE {
        let added = current_line.attempt_to_add_word(&current_word);
        if !added {
          lines.push(current_line);
          current_line = Line::new(
            self.meta.space_width(),
            text.font_size(),
            text.max_line_size(),
          );
          current_line.attempt_to_add_word(&current_word);
        }
        current_word = Word::new(text.font_size());

        lines.push(current_line);
        current_line = Line::new(
          self.meta.space_width(),
          text.font_size(),
          text.max_line_size(),
        );
        current_line.attempt_to_add_word(&current_word);
        current_word = Word::new(text.font_size());
        continue;
      }

      let probably_character = self.meta.get_character(ascii);
      if let Some(character) = probably_character {
        current_word.add_character(character.clone());
      }
    }
    self.complete_structure(&mut lines, current_line, current_word, text);

    lines
  }

  fn complete_structure(
    &self,
    lines: &mut Vec<Line>,
    mut current_line: Line,
    current_word: Word,
    text: &GuiText,
  ) {
    let added = current_line.attempt_to_add_word(&current_word);
    if !added {
      lines.push(current_line);
      current_line = Line::new(
        self.meta.space_width(),
        text.font_size(),
        text.max_line_size(),
      );
      current_line.attempt_to_add_word(&current_word);
    }
    lines.push(current_line);
  }

  fn create_quad_verticies(&self, text: &mut GuiText, lines: Vec<Line>) -> TextMeshData {
    text.set_number_of_line(lines.len() as i32);
    let mut curser_x = 0.0;
    let mut curser_y = 0.0;
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    for line in lines {
      if text.is_centered() {
        curser_x = (line.max_length() - line.line_length()) * 0.5;
      }
      for word in line.words() {
        for letter in word.characters() {
          self.add_verticies_for_character(
            curser_x,
            curser_y,
            &letter,
            text.font_size(),
            &mut verticies,
          );
          self.add_uv_coords(
            &mut uvs,
            letter.x_coord(),
            letter.y_coord(),
            letter.x_max_coord(),
            letter.y_max_coord(),
          );
          curser_x += letter.x_advance() * text.font_size();
        }
        curser_x += self.meta.space_width() * text.font_size();
      }
      curser_x = 0.0;
      curser_y += LINE_HEIGHT * text.font_size();
    }

    TextMeshData::new(verticies, uvs, text.colour())
  }

  fn add_verticies_for_character(
    &self,
    curser_x: f32,
    curser_y: f32,
    character: &Character,
    font_size: f32,
    verticies: &mut Vec<[f32; 2]>,
  ) {
    let x = curser_x + character.x_offset() * font_size;
    let y = curser_y + character.y_offset() * font_size;
    let max_x = x + character.size_x() * font_size;
    let max_y = y + character.size_y() * font_size;
    let proper_x = (2.0 * x) - 1.0;
    let proper_y = (-2.0 * y) + 1.0;
    let proper_max_x = (2.0 * max_x) - 1.0;
    let proper_max_y = (-2.0 * max_y) + 1.0;
    self.add_verticies(verticies, proper_x, proper_y, proper_max_x, proper_max_y);
  }

  fn add_verticies(&self, verticies: &mut Vec<[f32; 2]>, x: f32, y: f32, max_x: f32, max_y: f32) {
    verticies.push([x, y]);
    verticies.push([x, max_y]);
    verticies.push([max_x, max_y]);
    verticies.push([max_x, max_y]);
    verticies.push([max_x, y]);
    verticies.push([x, y]);
  }

  fn add_uv_coords(&self, uv_coords: &mut Vec<[f32; 2]>, x: f32, y: f32, max_x: f32, max_y: f32) {
    uv_coords.push([x, y]);
    uv_coords.push([x, max_y]);
    uv_coords.push([max_x, max_y]);
    uv_coords.push([max_x, max_y]);
    uv_coords.push([max_x, y]);
    uv_coords.push([x, y]);
  }
}

impl GuiText {
  pub fn new(
    text: String,
    font_size: f32,
    position: Vec2,
    colour: Vec4,
    max_line_length: f32,
    centered: bool,
  ) -> GuiText {
    let gui_text = GuiText {
      text,
      font_size,

      descriptor_set: None,
      colour,
      position,
      max_line_size: max_line_length,
      number_of_lines: 0,

      //font,
      center_text: centered,
    };

    gui_text
  }

  pub fn set_colour(&mut self, colour: Vec4) {
    self.colour = colour;
  }

  pub fn set_position(&mut self, pos: Vec2) {
    self.position = pos;
  }

  pub fn colour(&self) -> Vec4 {
    self.colour
  }

  pub fn number_of_lines(&self) -> i32 {
    self.number_of_lines
  }

  pub fn position(&self) -> Vec2 {
    self.position
  }

  pub fn descriptor_set(&self) -> &Option<DescriptorSet> {
    &self.descriptor_set
  }

  pub fn font_size(&self) -> f32 {
    self.font_size
  }

  pub fn set_number_of_line(&mut self, lines: i32) {
    self.number_of_lines = lines;
  }

  pub fn is_centered(&self) -> bool {
    self.center_text
  }

  pub fn max_line_size(&self) -> f32 {
    self.max_line_size
  }

  pub fn text(&self) -> String {
    self.text.to_string()
  }
}

impl FontType {
  pub fn new(file: String, sampler: &Sampler, vulkan: &mut Vulkan) -> FontType {
    let descriptor_pool = DescriptorPoolBuilder::new()
      .num_combined_image_samplers(1)
      .build(vulkan.device());

    let image = image::open(file.to_owned() + ".png")
      .expect(&("Failed to load font: ".to_string() + &file))
      .fliph()
      .to_rgba8();

    let font_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
    let font_descriptor_set = DescriptorSet::builder()
      .combined_image_sampler_fragment()
      .build(vulkan.device(), &descriptor_pool);
    let font_descriptor_set_writer =
      DescriptorWriter::builder().update_image(&font_texture, &sampler, &font_descriptor_set);

    font_descriptor_set_writer.build(vulkan.device());

    let text_vertex = TextVertex {
      pos: [0.0, 0.0, 0.0, 0.0],
      uv: [0.0, 0.0, 0.0, 0.0],
      colour: [1.0, 1.0, 1.0, 1.0],
    };

    let graphics_pipeline_builder = GraphicsPipelineBuilder::new()
      .topology_triangle_list()
      .front_face_counter_clockwise()
      .polygon_mode_fill()
      .samples_1();

    let layouts = vec![font_descriptor_set.layouts()[0]];

    let shader = Shader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../../shaders/new_text_vert.spv")[..]),
      Cursor::new(&include_bytes!("../../shaders/new_text_frag.spv")[..]),
      text_vertex,
      vec![
        offset_of!(TextVertex, pos) as u32,
        offset_of!(TextVertex, uv) as u32,
        offset_of!(TextVertex, colour) as u32,
      ],
      &graphics_pipeline_builder,
      vulkan.texture_renderpass(),
      vulkan.viewports(),
      vulkan.scissors(),
      &layouts,
      None as Option<(i32, Vec<u32>)>,
    );

    let loader = TextMeshCreator::new(file);

    FontType {
      texture: font_texture,
      pool: descriptor_pool,
      descriptor: font_descriptor_set,
      shader,
      loader,
    }
  }

  pub fn shader(&self) -> &Shader<TextVertex> {
    &self.shader
  }

  pub fn descriptor(&self) -> &DescriptorSet {
    &self.descriptor
  }

  pub fn texture(&self) -> &vkImage {
    &self.texture
  }

  pub fn load_text(&mut self, text: &mut GuiText) -> TextMeshData {
    self.loader.create_text_data(text)
  }
}

impl Line {
  pub fn new(space_width: f32, font_size: f32, max_length: f32) -> Line {
    Line {
      space_size: space_width * font_size,
      max_length,
      words: Vec::new(),
      current_line_length: 0.0,
    }
  }

  pub fn attempt_to_add_word(&mut self, word: &Word) -> bool {
    let mut additional_length = word.word_width();
    additional_length += if !self.words.is_empty() {
      self.space_size
    } else {
      0.0
    };
    if self.current_line_length + additional_length <= self.max_length
      || self.current_line_length == 0.0
    {
      self.words.push(word.clone());
      self.current_line_length += additional_length;
      return true;
    }

    false
  }

  pub fn max_length(&self) -> f32 {
    self.max_length
  }

  pub fn line_length(&self) -> f32 {
    self.current_line_length
  }

  pub fn words(&self) -> Vec<Word> {
    self.words.clone()
  }
}

impl Word {
  pub fn new(font_size: f32) -> Word {
    Word {
      width: 0.0,
      font_size,
      characters: Vec::new(),
    }
  }

  pub fn add_character(&mut self, c: Character) {
    self.width += c.x_advance() * self.font_size;
    self.characters.push(c);
  }

  pub fn characters(&self) -> Vec<Character> {
    self.characters.clone()
  }

  pub fn word_width(&self) -> f32 {
    self.width
  }
}

impl TextMeshData {
  pub fn new(
    positions: Vec<[f32; 2]>,
    texture_coords: Vec<[f32; 2]>,
    colour: Vec4,
  ) -> TextMeshData {
    let mut data = Vec::new();
    for (pos, uv) in positions.iter().zip(texture_coords.iter()) {
      data.push(TextVertex {
        pos: [pos[0], pos[1], -1.0, 1.0],
        uv: [1.0 - uv[0], uv[1], 0.0, 0.0],
        colour: [colour.x, colour.y, colour.z, colour.w],
      });
    }

    TextMeshData { data }
  }

  pub fn data(&self) -> &Vec<TextVertex> {
    &self.data
  }
}

impl Meta {
  pub fn load_font_data(file: String) -> Meta {
    let file = File::open(file.to_owned() + ".fnt").unwrap();
    let buffer_reader = BufReader::new(file);

    let mut meta = Meta {
      character_data: HashMap::new(),

      values: HashMap::new(),
      buffer: buffer_reader,
      padding: Vec::new(),
      padding_width: 0,
      padding_height: 0,
      vert_per_pixel_size: 0.0,
      horz_per_pixel_size: 0.0,

      space_width: 0.0,
    };

    meta.load_padding_data();
    meta.load_line_sizes();
    let image_width = meta.get_values_of_variables("scaleW")[0];
    meta.load_character_data(image_width);

    meta
  }

  pub fn get_character(&self, id: i32) -> Option<&Character> {
    self.character_data.get(&id)
  }

  pub fn process_next_line(&mut self) -> bool {
    let mut line = format!("");
    if let Ok(amount_of_data) = self.buffer.read_line(&mut line) {
      for part in line.split(' ') {
        let value_pairs: Vec<String> = part.split('=').map(|s| s.to_string()).collect();
        if value_pairs.len() == 2 {
          self
            .values
            .insert(value_pairs[0].clone(), value_pairs[1].clone());
        }
      }
      if amount_of_data == 0 {
        false
      } else {
        true
      }
    } else {
      false
    }
  }

  pub fn get_values_of_variables(&mut self, var: &str) -> Vec<f32> {
    let mut actual_values: Vec<f32> = Vec::new();

    if let Some(string_numbers) = self.values.get(var) {
      let values: Vec<String> = string_numbers.split(',').map(|s| s.to_string()).collect();
      for i in 0..values.len() {
        actual_values.push(values[i].parse().unwrap());
      }
    }

    actual_values
  }

  pub fn load_padding_data(&mut self) {
    self.process_next_line();
    self.padding = self
      .get_values_of_variables("padding")
      .iter()
      .map(|p| *p as i32)
      .collect::<Vec<i32>>();
    self.padding_width = (self.padding[PAD_LEFT] + self.padding[PAD_RIGHT]) as i32;
    self.padding_height = (self.padding[PAD_TOP] + self.padding[PAD_BOTTOM]) as i32;
  }

  pub fn load_line_sizes(&mut self) {
    self.process_next_line();
    let line_height_pixels =
      self.get_values_of_variables("lineHeight")[0] - self.padding_height as f32;
    self.vert_per_pixel_size = LINE_HEIGHT / line_height_pixels as f32;
    self.horz_per_pixel_size = self.vert_per_pixel_size / 1.0;
  }

  pub fn load_character_data(&mut self, image_width: f32) {
    self.process_next_line();
    self.process_next_line();

    while self.process_next_line() {
      let c = self.load_character(image_width);
      if let Some(character) = c {
        self.character_data.insert(character.id(), character);
      }
    }
  }

  pub fn load_character(&mut self, image_size: f32) -> Option<Character> {
    let id = self.get_values_of_variables("id")[0] as i32;
    if id == SPACE_ASCII {
      self.space_width = (self.get_values_of_variables("xadvance")[0] - self.padding_width as f32)
        * self.horz_per_pixel_size;
      return None;
    }

    let x_tex = (self.get_values_of_variables("x")[0]
      + (self.padding[PAD_LEFT] as f32 - DESIRED_PADDING as f32))
      / image_size;
    let y_tex = (self.get_values_of_variables("y")[0]
      + (self.padding[PAD_TOP] as f32 - DESIRED_PADDING as f32))
      / image_size;
    let width = self.get_values_of_variables("width")[0]
      - (self.padding_width as f32 - (2.0 * DESIRED_PADDING as f32));
    let height = self.get_values_of_variables("height")[0]
      - ((self.padding_height as f32) - (2.0 * DESIRED_PADDING as f32));

    let quad_width = width * self.horz_per_pixel_size;
    let quad_height = height * self.vert_per_pixel_size;

    let x_tex_size = width / image_size;
    let y_tex_size = height / image_size;

    let x_off = (self.get_values_of_variables("xoffset")[0] + self.padding[PAD_LEFT] as f32
      - DESIRED_PADDING as f32)
      * self.horz_per_pixel_size;
    let y_off = (self.get_values_of_variables("yoffset")[0]
      + (self.padding[PAD_TOP] as f32 - DESIRED_PADDING as f32))
      * self.vert_per_pixel_size;
    let x_advance = (self.get_values_of_variables("xadvance")[0] - self.padding_width as f32)
      * self.horz_per_pixel_size;

    return Some(Character::new(
      id,
      x_tex,
      y_tex,
      x_tex_size,
      y_tex_size,
      x_off,
      y_off,
      quad_width,
      quad_height,
      x_advance,
    ));
  }

  pub fn space_width(&self) -> f32 {
    self.space_width
  }
}

impl Character {
  pub fn new(
    id: i32,
    x_coord: f32,
    y_coord: f32,
    x_text_size: f32,
    y_text_size: f32,
    x_offset: f32,
    y_offset: f32,
    size_x: f32,
    size_y: f32,
    x_advance: f32,
  ) -> Character {
    Character {
      id,
      x_coord,
      y_coord,
      x_offset,
      y_offset,
      size_x,
      size_y,
      x_max_coord: x_text_size + x_coord,
      y_max_coord: y_text_size + y_coord,
      x_advance,
    }
  }

  pub fn id(&self) -> i32 {
    self.id
  }

  pub fn x_coord(&self) -> f32 {
    self.x_coord
  }

  pub fn y_coord(&self) -> f32 {
    self.y_coord
  }

  pub fn x_max_coord(&self) -> f32 {
    self.x_max_coord
  }

  pub fn y_max_coord(&self) -> f32 {
    self.y_max_coord
  }

  pub fn x_offset(&self) -> f32 {
    self.x_offset
  }

  pub fn y_offset(&self) -> f32 {
    self.y_offset
  }

  pub fn size_x(&self) -> f32 {
    self.size_x
  }

  pub fn size_y(&self) -> f32 {
    self.size_y
  }

  pub fn x_advance(&self) -> f32 {
    self.x_advance
  }
}

impl PartialEq for Character {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }

  fn ne(&self, other: &Self) -> bool {
    self.id != other.id
  }
}

impl Eq for Character {}

impl PartialEq for FontType {
  fn eq(&self, other: &Self) -> bool {
    self.loader == other.loader
  }

  fn ne(&self, other: &Self) -> bool {
    self.loader != other.loader
  }
}

impl Eq for FontType {}

impl PartialEq for Meta {
  fn eq(&self, other: &Self) -> bool {
    self.character_data == other.character_data
  }

  fn ne(&self, other: &Self) -> bool {
    self.character_data == other.character_data
  }
}

impl Eq for Meta {}

impl PartialEq for GuiText {
  fn eq(&self, other: &Self) -> bool {
    self.text == other.text && self.font_size == other.font_size
  }

  fn ne(&self, other: &Self) -> bool {
    self.text != other.text || self.font_size != other.font_size
  }
}

//
//#[derive(Clone)]
//pub struct Glyph {
//  top: f32,
//  left: f32,
//  width: f32,
//  height: f32,
//  x_offset: f32,
//  y_offset: f32,
//  x_advance: f32,
//  uv_bot: f32,
//  uv_left: f32,
//  uv_width: f32,
//  uv_height: f32,
//}
//
//#[derive(Clone)]
//pub struct GlyphCache {
//  average_advance: f32,
//  line_height: f32,
//  font_size: i8,
//  glyphs: HashMap<char, Glyph>,
//}
//
//pub struct Font {
//  glyph_cache: GlyphCache,
//  texture: vkImage,
//  descriptor_set: DescriptorSet,
//  descriptor_pool: vk::DescriptorPool,
//}
//
//impl Glyph {
//  pub fn load(location: String) -> GlyphCache {
//    let file = File::open(location.to_owned() + ".fnt").unwrap();
//    let buffer_reader = BufReader::new(file);
//
//    let mut glyphs = HashMap::new();
//
//    let mut average_advance: f32 = 0.0;
//
//    let mut line_height = 0.0;
//
//    let mut scale_w = 1.0;
//    let mut scale_h = 1.0;
//
//    let mut font_size = 1;
//
//    for line in buffer_reader.lines() {
//      let line = line.unwrap();
//      let mut segments: Vec<&str> = line.split(' ').filter(|s| *s != "").collect();
//
//      let segment_0 = segments.remove(0);
//
//      if segment_0.contains("char") && !segments[0].contains("count") {
//        let idx = Font::value_from_string_pair(segments.remove(0).to_string()) as u8;
//
//        let x = Font::value_from_string_pair(segments[0].to_string()) as f32;
//        let y = Font::value_from_string_pair(segments[1].to_string()) as f32;
//        let width = Font::value_from_string_pair(segments[2].to_string()) as f32;
//        let height = Font::value_from_string_pair(segments[3].to_string()) as f32;
//        let x_offset = Font::value_from_string_pair(segments[4].to_string()) as f32;
//        let y_offset = Font::value_from_string_pair(segments[5].to_string()) as f32;
//        let x_advance = width / (Font::value_from_string_pair(segments[6].to_string()) as f32);
//        let page = Font::value_from_string_pair(segments[7].to_string()) as u32;
//
//        average_advance += x_advance;
//
//        println!("{}: {}", idx as char, height);
//
//        glyphs.insert(
//          idx as char,
//          Glyph {
//            top: y_offset as f32 / scale_h,
//            left: x_offset / scale_w,
//            width: width / scale_w,
//            height: height / scale_h,
//            x_offset: x_offset / scale_w,
//            y_offset: y_offset / scale_h,
//            x_advance: x_advance / scale_w,
//            uv_bot: y / scale_h,
//            uv_left: x / scale_w,
//            uv_width: width / scale_w,
//            uv_height: height / scale_h,
//          },
//        );
//      } else if segment_0.contains("common") {
//        if segments[0].contains("lineHeight") {
//          line_height = Font::value_from_string_pair(segments[2].to_string()) as f32;
//        }
//
//        if segments[2].contains("scaleW") {
//          scale_w = Font::value_from_string_pair(segments[2].to_string()) as f32;
//        }
//        if segments[3].contains("scaleH") {
//          scale_h = Font::value_from_string_pair(segments[3].to_string()) as f32;
//        }
//      } else if segment_0.contains("info") {
//        if segments[0].contains("size") {
//          font_size = Font::value_from_string_pair(segments[1].to_string()) as i8;
//        }
//      }
//    }
//
//    average_advance /= glyphs.len() as f32;
//
//    GlyphCache::new(average_advance, line_height, font_size, glyphs)
//  }
//}
//
//impl GlyphCache {
//  pub fn new(
//    average_advance: f32,
//    line_height: f32,
//    font_size: i8,
//    glyphs: HashMap<char, Glyph>,
//  ) -> GlyphCache {
//    GlyphCache {
//      average_advance,
//      line_height,
//      font_size,
//      glyphs,
//    }
//  }
//
//  pub fn glyphs(&self) -> &HashMap<char, Glyph> {
//    &self.glyphs
//  }
//}
//
//impl Font {
//  pub fn new<T: Into<String>>(vulkan: &mut Vulkan, sampler: &Sampler, location: T) -> Font {
//    let location = location.into();
//    let glyphs = Glyph::load(location.clone());
//
//    let image = image::open(location.to_owned() + ".png")
//      .expect(&("Failed to load font: ".to_string() + &location))
//      .fliph()
//      .to_rgba8();
//
//    let descriptor_pool = DescriptorPoolBuilder::new()
//      .num_combined_image_samplers(1)
//      .build(vulkan.device());
//
//    let font_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
//    let font_descriptor_set = DescriptorSet::builder()
//      .combined_image_sampler_fragment()
//      .build(vulkan.device(), &descriptor_pool);
//    let font_descriptor_set_writer =
//      DescriptorWriter::builder().update_image(&font_texture, &sampler, &font_descriptor_set);
//
//    font_descriptor_set_writer.build(vulkan.device());
//
//    Font {
//      glyph_cache: glyphs,
//      texture: font_texture,
//      descriptor_set: font_descriptor_set,
//      descriptor_pool,
//    }
//  }
//
//  pub fn generate_letter_draws(
//    &mut self,
//    text_size: f32,
//    text: String,
//  ) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
//    let mut text_data = Vec::new();
//
//    let mut pos_x = 0.0;
//    let mut pos_y = 0.0;
//
//    for c in text.chars() {
//      if let Some(char_info) = self.glyph_cache.glyphs().get(&c) {
//        let x = pos_x + char_info.left - char_info.x_offset;
//        let y = char_info.top - char_info.y_offset;
//
//        let width = char_info.left + char_info.width;
//        let height = char_info.top + char_info.height - char_info.y_offset;
//
//        let uvx = char_info.uv_left;
//        let uvxw = char_info.uv_left + char_info.uv_width;
//        let uvy = char_info.uv_bot;
//        let uvyh = char_info.uv_bot + char_info.uv_height;
//
//        text_data.push((
//          x,
//          y,
//          width,
//          height,
//          1.0 - uvx,
//          uvyh,
//          1.0 - uvxw,
//          uvy,
//          char_info.x_offset,
//          char_info.y_offset,
//        ));
//        pos_x += char_info.width + char_info.x_advance;
//
//        //if char_info.width == 0.0 || char_info.height == 0.0 {
//        //  pos_x += text_size;
//        //  continue;
//        //}
//
//        //let w_h_ratio = char_info.width / char_info.height;
//
//        //let height = text_size;
//        //let width = text_size * w_h_ratio;
//
//        //let x = pos_x + char_info.x_offset * width;
//        //let y = (1.0 - char_info.y_offset) * height - height;
//
//        //let uv_x0 = char_info.x;
//        //let uv_y0 = 1.0 - char_info.y;
//        //let uv_x1 = char_info.x + char_info.width;
//        //let uv_y1 = (1.0 - char_info.y) - char_info.height;
//
//        //text_data.push((
//        //  x,
//        //  y,
//        //  width,
//        //  height,
//        //  1.0 - uv_x0,
//        //  1.0 - uv_y1,
//        //  1.0 - uv_x1,
//        //  1.0 - uv_y0,
//        //));
//
//        //pos_x += char_info.x_advance * width;
//      }
//    }
//
//    text_data
//  }
//
//  fn value_from_string_pair(string: String) -> i32 {
//    string
//      .split('=')
//      .collect::<Vec<&str>>()
//      .last()
//      .unwrap()
//      .parse::<i32>()
//      .unwrap()
//  }
//
//  pub fn get_font_data(&self) -> GlyphCache {
//    self.glyph_cache.clone()
//  }
//
//  pub fn descriptor(&self) -> &DescriptorSet {
//    &self.descriptor_set
//  }
//}
//#[derive(Clone)]
//pub struct FontChar {
//  pub x: f32,
//  pub y: f32,
//  pub width: f32,
//  pub height: f32,
//  pub x_offset: f32,
//  pub y_offset: f32,
//  pub x_advance: f32, // not used?
//  pub page: u32,
//}
//
//impl FontChar {
//  pub fn new_empty() -> FontChar {
//    FontChar {
//      x: 0.0,
//      y: 0.0,
//      width: 0.0,
//      height: 0.0,
//      x_offset: 0.0,
//      y_offset: 0.0,
//      x_advance: 0.0,
//      page: 0,
//    }
//  }
//}
//
//pub struct Font {
//  chars: Vec<FontChar>,
//  texture: vkImage,
//  descriptor_set: DescriptorSet,
//  descriptor_pool: vk::DescriptorPool,
//
//  width: u32,
//  height: u32,
//  // line height of font
//  pub line_height: u32,
//
//  // size of font (width)
//  pub size: u32,
//
//  pub min_offset_y: f32,
//  pub avg_xadvance: f32,
//}
//
//impl Font {
//  pub fn new<T: Into<String>>(vulkan: &mut Vulkan, sampler: &Sampler, location: T) -> Font {
//    Font::load_font(vulkan, sampler, location)
//  }
//
//  fn load_font<T: Into<String>>(vulkan: &mut Vulkan, sampler: &Sampler, location: T) -> Font {
//    let location = &location.into();
//
//    let image = image::open(location.to_owned() + ".png")
//      .expect(&("Failed to load font: ".to_string() + location))
//      .fliph()
//      .to_rgba8();
//    let image_width = image.width() as f32;
//    let image_height = image.height() as f32;
//
//    let mut min_off_y = 100000.0;
//    let mut xadvance_sum = 0.0;
//
//    let mut line_height = 1;
//
//    let mut font_chars = Vec::new();
//
//    let file = File::open(location.to_owned() + ".fnt").unwrap();
//    let buffer_reader = BufReader::new(file);
//
//    let mut num_chars = 0;
//    for line in buffer_reader.lines() {
//      let line = line.unwrap();
//      let mut segments: Vec<&str> = line.split(' ').filter(|s| *s != "").collect();
//
//      let segment_0 = segments.remove(0);
//
//      if segment_0.contains("char") && !segments[0].contains("count") {
//        let idx = Font::value_from_string_pair(segments.remove(0).to_string()) as usize;
//
//        while idx + 1 > font_chars.len() {
//          font_chars.push(FontChar::new_empty());
//        }
//
//        let x = Font::value_from_string_pair(segments[0].to_string()) as f32 / 512.0;
//        let y = Font::value_from_string_pair(segments[1].to_string()) as f32 / 512.0;
//        let width = Font::value_from_string_pair(segments[2].to_string()) as f32 / 512.0;
//        let height = Font::value_from_string_pair(segments[3].to_string()) as f32 / 512.0;
//        let x_offset = Font::value_from_string_pair(segments[4].to_string()) as f32 / 512.0;
//        let y_offset = Font::value_from_string_pair(segments[5].to_string()) as f32 / 512.0;
//        let x_advance =
//          width / (Font::value_from_string_pair(segments[6].to_string()) as f32 / 512.0);
//        let page = Font::value_from_string_pair(segments[7].to_string()) as u32;
//
//        if y_offset < min_off_y {
//          min_off_y = y_offset;
//        }
//        xadvance_sum += x_advance as f32;
//
//        font_chars[idx] = FontChar {
//          x,
//          y,
//          width,
//          height,
//          x_offset,
//          y_offset,
//          x_advance, // not used?
//          page,
//        };
//
//        num_chars += 1;
//      } else if segment_0.contains("common") {
//        line_height = Font::value_from_string_pair(segments[0].to_string()) as u32;
//      }
//    }
//
//    let avg_advance = xadvance_sum / num_chars as f32;
//
//    let descriptor_pool = DescriptorPoolBuilder::new()
//      .num_combined_image_samplers(1)
//      .build(vulkan.device());
//
//    let font_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
//    let font_descriptor_set = DescriptorSet::builder()
//      .combined_image_sampler_fragment()
//      .build(vulkan.device(), &descriptor_pool);
//    let font_descriptor_set_writer =
//      DescriptorWriter::builder().update_image(&font_texture, &sampler, &font_descriptor_set);
//
//    font_descriptor_set_writer.build(vulkan.device());
//
//    Font {
//      chars: font_chars,
//      texture: font_texture,
//      descriptor_set: font_descriptor_set,
//      descriptor_pool,
//
//      width: image_width as u32,
//      height: image_height as u32,
//
//      line_height,
//      size: 71, // TODO: Load from file
//
//      min_offset_y: min_off_y,
//      avg_xadvance: avg_advance,
//    }
//  }
//
//  pub fn get_font_data(&self) -> (Vec<FontChar>, u32, u32) {
//    (self.chars.clone(), self.line_height, self.size)
//  }
//
//  pub fn descriptor(&self) -> &DescriptorSet {
//    &self.descriptor_set
//  }
//
//  fn value_from_string_pair(string: String) -> i32 {
//    string
//      .split('=')
//      .collect::<Vec<&str>>()
//      .last()
//      .unwrap()
//      .parse::<i32>()
//      .unwrap()
//  }
//
//  pub fn generate_letter_draws(
//    &mut self,
//    text_size: f32,
//    text: String,
//  ) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32)> {
//    let mut text_data = Vec::new();
//
//    let mut pos_x = 0.0;
//
//    for c in text.chars() {
//      let char_info = &mut self.chars[c as i32 as usize];
//
//      if char_info.width == 0.0 || char_info.height == 0.0 {
//        pos_x += text_size;
//        continue;
//      }
//
//      let w_h_ratio = char_info.width / char_info.height;
//
//      let height = text_size;
//      let width = text_size * w_h_ratio;
//
//      let x = pos_x + char_info.x_offset * width;
//      let y = (1.0 - char_info.y_offset) * height - height;
//
//      let uv_x0 = char_info.x;
//      let uv_y0 = 1.0 - char_info.y;
//      let uv_x1 = char_info.x + char_info.width;
//      let uv_y1 = (1.0 - char_info.y) - char_info.height;
//
//      text_data.push((
//        x,
//        y,
//        width,
//        height,
//        1.0 - uv_x0,
//        1.0 - uv_y1,
//        1.0 - uv_x1,
//        1.0 - uv_y0,
//      ));
//
//      pos_x += char_info.x_advance * width;
//    }
//
//    text_data
//  }
//}
