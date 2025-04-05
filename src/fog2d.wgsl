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
    vision_range: f32,     // 视野范围 / vision range
    vision_falloff: f32,   // 视野衰减系数 / vision falloff coefficient
};

@group(0) @binding(0)
var<uniform> fog_material: FogMaterial;

@group(0) @binding(1)
var noise_texture: texture_2d<f32>;

@group(0) @binding(2)
var noise_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // 常量设置
    // Constants setting
    let inner_radius = 0.2;  // 内圈（完全透明区域）
    let outer_radius = 0.3;  // 外圈（开始迷雾区域）
    
    // 初始化迷雾颜色
    // Initialize fog color
    var fog_color = fog_material.color;
    
    // 计算到屏幕中心的距离
    // Calculate distance to screen center
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center);
    
    // 逐渐过渡透明度计算
    // Gradual transparency transition calculation
    var alpha = 1.0;
    
    if (dist < inner_radius) {
        // 在内圈内部 - 完全透明
        // Inside inner circle - completely transparent
        alpha = 0.0;
    } else if (dist < outer_radius) {
        // 在过渡区域 - 平滑过渡
        // In transition area - smooth transition
        alpha = smoothstep(inner_radius, outer_radius, dist);
    }
    
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

    // 返回最终颜色，使用计算的透明度
    // Return final color with calculated transparency
    return vec4<f32>(fog_color.rgb, alpha);
}
