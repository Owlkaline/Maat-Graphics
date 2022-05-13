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
  colour: Option<Vec4>,
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
  coloured_words: HashMap<usize, Vec4>,
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
      if self.text[i].0 == text {
        self.text[i].0.set_position(text.position());
        self.text[i].0.set_colour(text.colour());
        already_exists = true;
        break;
      }
    }

    for i in 0..self.unused_text.len() {
      if self.unused_text[i].0 .0 == text {
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
      if self.text[i].0 == text && self.text[i].0.position().x == text.position().x {
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
    let mut colours = Vec::new();

    let coloured_words = text.coloured_words();
    let mut current_word: usize = 0;

    for line in lines {
      if text.is_centered() {
        curser_x = (line.max_length() - line.line_length()) * 0.5;
      }
      for word in line.words() {
        let mut actual_word = false;

        let colour = {
          if let Some(w_colour) = coloured_words.get(&current_word) {
            *w_colour
          } else {
            text.colour()
          }
        };

        for letter in word.characters() {
          if letter.id() != '\n' as i32 && letter.id() != ' ' as i32 {
            actual_word = true;
          }
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
          self.add_colour(&mut colours, colour);
          curser_x += letter.x_advance() * text.font_size();
        }
        curser_x += self.meta.space_width() * text.font_size();

        if actual_word {
          current_word += 1;
        }
      }
      curser_x = 0.0;
      curser_y += LINE_HEIGHT * text.font_size();
    }

    TextMeshData::new(verticies, uvs, colours) //text.colour())
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

  fn add_colour(&self, colours: &mut Vec<[f32; 4]>, colour: Vec4) {
    // for each vertex
    for _ in 0..6 {
      colours.push([colour.x, colour.y, colour.z, colour.w]);
    }
  }
}

impl GuiText {
  pub fn new(
    text: String,
    font_size: f32,
    position: Vec2,
    colour: Vec4,
    coloured_words: HashMap<usize, Vec4>,
    max_line_length: f32,
    centered: bool,
  ) -> GuiText {
    let gui_text = GuiText {
      text,
      font_size,

      descriptor_set: None,
      colour,
      coloured_words,
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

  pub fn coloured_words(&self) -> &HashMap<usize, Vec4> {
    &self.coloured_words
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
    if self.words.len() == 2 {
      //print!(
      //  "{:?}",
      //  self.words[0]
      //    .characters()
      //    .iter()
      //    .map(|c| c.id())
      //    .collect::<Vec<i32>>()
      //);
      //println!(
      //  "{:?}",
      //  self.words[1]
      //    .characters()
      //    .iter()
      //    .map(|c| c.id())
      //    .collect::<Vec<i32>>()
      //);
    }
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
      colour: None,
    }
  }

  pub fn add_colour(&mut self, colour: Vec4) {
    self.colour = Some(colour);
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

  pub fn colour(&self) -> Option<Vec4> {
    self.colour
  }
}

impl TextMeshData {
  pub fn new(
    positions: Vec<[f32; 2]>,
    texture_coords: Vec<[f32; 2]>,
    colours: Vec<[f32; 4]>,
  ) -> TextMeshData {
    let mut data = Vec::new();
    for (pos, uv, colour) in (positions.iter().zip(texture_coords.iter()))
      .zip(colours.iter())
      .map(|((p, u), c)| (p, u, c))
    {
      data.push(TextVertex {
        pos: [pos[0], pos[1], -1.0, 1.0],
        uv: [1.0 - uv[0], uv[1], 0.0, 0.0],
        colour: *colour,
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
    self.text == other.text
      && self.font_size == other.font_size
      && self.colour == other.colour
      && self.position == other.position
      && self.font_size == other.font_size
  }

  fn ne(&self, other: &Self) -> bool {
    self.text != other.text
      || self.font_size != other.font_size && self.colour != other.colour
      || self.position != other.position
      || self.font_size != other.font_size
  }
}
