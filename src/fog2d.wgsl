#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 迷雾设置结构
// Fog settings structure
struct FogMaterial {
    color: vec4<f32>,       // 迷雾颜色 / fog color
    use_noise: u32,        // 是否使用噪声纹理 / whether to use noise texture
    noise_intensity: f32,  // 噪声强度 / noise intensity
    noise_scale: f32,      // 噪声缩放 / noise scale
    noise_speed: f32,      // 噪声速度 / noise speed
    time: f32             // 当前时间 / current time
};

@group(0) @binding(0)
var<uniform> fog_material: FogMaterial;

@group(0) @binding(1)
var noise_texture: texture_2d<f32>;

@group(0) @binding(2)
var noise_sampler: sampler;

// 添加可见性纹理绑定
// Add visibility texture binding
@group(0) @binding(3)
var visibility_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // 初始化迷雾颜色
    // Initialize fog color
    var fog_color = fog_material.color;
    
    // 采样可见性纹理获取当前像素的可见性值
    // Sample visibility texture to get visibility value for current pixel
    let visibility = textureSample(visibility_texture, noise_sampler, in.uv).r;
    
    // 增强可见性对比度，使可见区域更清晰
    // Enhance visibility contrast to make visible areas clearer
    let enhanced_visibility = pow(visibility, 0.5); // 使用平方根增强低可见性区域
    
    // 根据可见性值调整透明度（可见区域透明，不可见区域显示迷雾）
    // Adjust alpha based on visibility (visible areas are transparent, fog appears in non-visible areas)
    var alpha = 1.0 - enhanced_visibility;  // 可见度越高，迷雾越透明 / higher visibility means more transparent fog
    
    // 确保透明度在有效范围内
    // Ensure alpha is in valid range
    alpha = clamp(alpha, 0.0, 1.0);
    
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
        
        // 使用噪声值进行插值
        // Use noise value for interpolation with intensity parameter
        let noise_value = noise.r * fog_material.noise_intensity;
        fog_color = vec4<f32>(mix(vec3<f32>(0.0, 0.0, 0.0), fog_color.rgb, noise_value), fog_color.a);
    }

    // 返回最终颜色，使用基于可见性计算的透明度
    // Return final color with visibility-based transparency
    return vec4<f32>(fog_color.rgb, alpha);
}
