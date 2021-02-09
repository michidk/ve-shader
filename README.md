# Shader Compiler for Vulkan Engine

![Continuous integration](https://github.com/michidk/ve-shader/workflows/Continuous%20Integration/badge.svg)

This utility compiles my custom glsl shader format to the SPIV-V format using shader-c.

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

Our custom format combines vertex, fragment, and geometry shader in one file. `//#` at the beginning of a line denotes that a custom instruction to this transpiler follows. While the commands NAME, AUTHOR, and DESCRIPTION are optional (they are not even parsed), TYPE will instruct this utility to compile the following to a shader of that type.

```glsl
//# NAME Vertex Color
//# AUTHOR Michael Lohr
//# DESCRIPTION Just renders the vertex colors, nothing else.

//# TYPE VERTEX
#version 450

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
#version 450

layout (location = 0) in vec4 i_color;
layout (location = 0) out vec4 o_color;

void main(){
    o_color = i_color;
}
```
