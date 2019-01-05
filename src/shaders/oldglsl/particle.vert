#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in mat4 model;
layout(location = 5) in vec4 offset; 
layout(location = 6) in vec2 blendFactor;

out vec2 textureCoords1;
out vec2 textureCoords2;
out float blend;
out float fade;

uniform mat4 projection;
uniform float numberOfRows;

void main() {
  
  vec2 textureCoords = position.xy;
  textureCoords.y *= -1;
  textureCoords.y = numberOfRows + textureCoords.y;
  textureCoords /= numberOfRows;
  textureCoords1 = textureCoords + offset.xy;
  textureCoords2 = textureCoords + offset.zw;
  blend = blendFactor.x;
  fade = blendFactor.y;
  
  gl_Position = projection * model * vec4(position.xy, 0.0f, 1.0f);
}
