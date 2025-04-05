struct VertexInput {
    @location(0) position: vec3<f32>, // <x, y, z>
    @location(1) color: vec4<f32>, // <r, g, b, a>
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>, // <x, y, z, w>
    @location(0) color: vec4<f32>, // <r, g, b, a>
}

struct CameraTransforms {
    world: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<storage, read> vertices: array<VertexInput>;

@group(0) @binding(1)
var<uniform> camera: CameraTransforms;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    // Compute the projected vertex position
    var output: VertexOutput;
    let worldview: mat4x4<f32> = camera.view * camera.world;
    output.position = camera.proj * worldview * vec4<f32>(vertices[index].position, 1.0);
    output.color = vertices[index].color;
    return output;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
