struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceData {
    @location(2) model_transform_0: vec4<f32>,
    @location(3) model_transform_1: vec4<f32>,
    @location(4) model_transform_2: vec4<f32>,
    @location(5) model_transform_3: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) layer: u32,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) layer: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceData,
) -> VertexOutput {
    var out: VertexOutput;
    let model_transform = mat4x4<f32>(
        instance.model_transform_0,
        instance.model_transform_1,
        instance.model_transform_2,
        instance.model_transform_3,
    );
    
    out.uv = model.uv;
    out.layer = instance.layer;
    out.color = instance.color;
    out.clip_position = camera.view_proj * model_transform * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var diffuse_textures: texture_2d_array<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let layer_index: i32 = i32(in.layer);

    let texture_color = textureSample(diffuse_textures, diffuse_sampler, in.uv, layer_index);
    return in.color * texture_color;
}