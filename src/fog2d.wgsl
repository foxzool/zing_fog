#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 迷雾设置结构
// Fog settings structure
struct FogMaterial {
    color: vec4<f32>,       // 迷雾颜色 / fog color
    use_noise: u32,        // 是否使用噪声纹理 / whether to use noise texture
    noise_intensity: f32,  // 噪声强度 / noise intensity
    noise_scale: f32,      // 噪声缩放 / noise scale
    noise_speed: f32,      // 噪声速度 / noise speed
    time: f32,            // 当前时间 / current time
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
        // 计算动态UV坐标，基于时间和噪声速度
        // Calculate dynamic UV coordinates based on time and noise speed
        var dynamic_uv = in.uv;
        
        // 如果噪声速度大于0，应用时间偏移
        // If noise speed is greater than 0, apply time offset
        if (fog_material.noise_speed > 0.0) {
            // 简单的UV动画，基于时间和速度
            // Simple UV animation based on time and speed
            dynamic_uv.x += sin(fog_material.time * fog_material.noise_speed * 0.1) * 0.1;
            dynamic_uv.y += cos(fog_material.time * fog_material.noise_speed * 0.1) * 0.1;
        }
        
        // 应用噪声缩放
        // Apply noise scale
        dynamic_uv = dynamic_uv * fog_material.noise_scale;
        
        // 采样噪声纹理
        // Sample noise texture
        let noise = textureSample(noise_texture, noise_sampler, dynamic_uv);
        
        // 使用噪声值进行插值：黑色区域保持黑色，白色区域显示配置的颜色
        // 应用噪声强度参数
        // Use noise value for interpolation with intensity parameter
        let noise_value = noise.r * fog_material.noise_intensity;
        let alpha = color.a;
        color = vec4<f32>(mix(vec3<f32>(0.0, 0.0, 0.0), color.rgb, noise_value), alpha);
    }
    
    return color;
}