struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceData {
    @location(2) position: vec3<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) rotation: f32,
    @location(5) layer: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceData,
) -> VertexOutput {
    var out: VertexOutput;

    

    out.uv = model.uv;

}