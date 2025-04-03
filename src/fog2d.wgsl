#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 迷雾设置结构
// Fog settings structure
struct FogMaterial {
    color: vec4<f32>,       // 迷雾颜色 / fog color
    use_noise: u32,        // 是否使用噪声纹理 / whether to use noise texture
};

@group(0) @binding(0)
var<uniform> fog_material: FogMaterial;

@group(0) @binding(1)
var noise_texture: texture_2d<f32>;

@group(0) @binding(2)
var noise_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var color = fog_material.color;
    
    // 确保迷雾始终不透明
    // Ensure fog is always opaque
    color.a = 1.0;
    
    // 如果启用了噪声纹理，则使用它来修改迷雾效果
    // If noise texture is enabled, use it to modify the fog effect
    if (fog_material.use_noise == 1u) {
        let noise = textureSample(noise_texture, noise_sampler, in.uv);
        
        // 使用噪声值进行插值：黑色区域保持黑色，白色区域显示配置的颜色
        // Use noise value for interpolation: black areas stay black, white areas show configured color
        let alpha = color.a;
        color = vec4<f32>(mix(vec3<f32>(0.0, 0.0, 0.0), color.rgb, noise.r), alpha);
    }
    
    return color;
}