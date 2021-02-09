# Shader Compiler for Vulkan Engine

![Continuous integration](https://github.com/michidk/ve-shader/workflows/Continuous%20Integration/badge.svg)

This utility compiles my custom GLSL shader format to the SPIR-V format using shader-c.

Made for [our game engine](https://github.com/michidk/vulkan-engine).

## Building

### Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Python 2](https://www.python.org/)
- Git
- cmake and ninja

Then build with `cargo build`.

## Execute

Get an overview of the parameters with `cargo run -- -h`.

For example, `cargo run -- ./shaders/*.glsl -o ./output` compiles all shaders in the `/shaders` folder and outputs the artifacts to the `/output` folder.


## Custom Format

Our custom format combines vertex, fragment, and geometry shader in one file.

### Instructions

`//#` at the beginning of a line denotes that a custom instruction follows. While the most instructions are optional, some are mandatory. One such instruction is `TYPE`, which will instruct this utility to compile the following code until the next type-instruction appears, to a shader of that type.

|Instruction|Required?|Arguments|Description|Example|
|--- | --- | --- | --- | --- |
|NAME|no|String|pretty formatted name of the shader||`//# NAME Phong Shader`|
|AUTHOR|no|String|author of the shader||`//# AUTHOR John Doe`|
|DESCRIPTION|no|String|describes what the shader does|`//# DESCRIPTION Applies the phong reflection model.`|
|VERSION|no|Version|adds `#version <version>` to each shader|`//# VERSION 450`|
|TYPE|yes|VERTEX,FRAGMENT,GEOMETRY|sets the type of the shader that follows|`//# TYPE VERTEX`|

### Example

```glsl
//# NAME Vertex Color
//# AUTHOR Michael Lohr
//# DESCRIPTION Renders the vertex colors

//# VERSION 450

//# TYPE VERTEX
layout (location = 0) in vec3 i_position;
layout (location = 1) in mat4 i_model_matrix;
layout (location = 5) in vec4 i_color;

layout (location = 0) out vec4 o_color;

void main() {
    gl_PointSize = 1.0;
    gl_Position = i_model_matrix * vec4(i_position, 1.0);

    o_color = i_color;
}

//# TYPE FRAGMENT
layout (location = 0) in vec4 i_color;
layout (location = 0) out vec4 o_color;

void main(){
    o_color = i_color;
}
```
