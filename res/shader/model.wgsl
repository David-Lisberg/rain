struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) layer: u32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) layer: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.color = model.color;
    out.layer = model.layer;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var diffuse_textures: texture_2d_array<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let layer_index: i32 = i32(in.layer);
    let sample = in.color * textureSample(diffuse_textures, diffuse_sampler, in.uv, layer_index);
    if sample.a < 0.01 {
        discard;
    }
    return sample;
}