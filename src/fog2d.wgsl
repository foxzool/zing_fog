#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 迷雾设置结构
// Fog settings structure
struct FogMaterial {
    color: vec4<f32>,       // 迷雾颜色 / fog color
};

@group(0) @binding(0)
var<uniform> fog_material: FogMaterial;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

    return fog_material.color;
}