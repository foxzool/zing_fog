#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 迷雾设置结构
// Fog settings structure
struct FogSettings {
    color: vec4<f32>,       // 迷雾颜色 / fog color
    center: vec2<f32>,     // 迷雾中心位置 / fog center position
    density: f32,          // 迷雾密度 / fog density
    range: f32,            // 迷雾范围 / fog range
    time: f32,             // 时间（用于动画）/ time (for animation)
    clear_radius: f32,     // 相机周围的透明半径 / clear radius around camera
    clear_falloff: f32,    // 边缘过渡效果 / edge falloff effect
};

@group(0) @binding(0)
var<uniform> fog_settings: FogSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

    return fog_settings.color;
}