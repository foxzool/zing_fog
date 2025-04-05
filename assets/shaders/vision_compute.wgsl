// 视野参数结构体
struct VisionParams {
    position: vec2<f32>,
    range: f32,
    falloff: f32,
}

// 视野参数数组
@group(0) @binding(0)
var<storage, read> vision_params: array<VisionParams>;

// 输出纹理
@group(0) @binding(1)
var output_texture: texture_storage_2d<r32float, write>;

// 计算着色器入口点
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dimensions = textureDimensions(output_texture);
    let coords = vec2<i32>(global_id.xy);
    
    // 检查坐标是否在纹理范围内
    if (coords.x >= dimensions.x || coords.y >= dimensions.y) {
        return;
    }
    
    // 将纹理坐标转换为世界空间坐标 (0.0 到 1.0)
    let uv = vec2<f32>(coords) / vec2<f32>(dimensions);
    
    // 初始化可见性为0（完全不可见）
    var visibility = 0.0;
    
    // 遍历所有视野提供者
    for (var i = 0u; i < arrayLength(&vision_params); i++) {
        let provider = vision_params[i];
        
        // 计算当前点到视野提供者的距离
        let dist = distance(uv, provider.position);
        
        // 如果在视野范围内，计算可见性
        if (dist <= provider.range) {
            // 使用平滑步进函数计算可见性
            let t = smoothstep(provider.range, provider.range - provider.falloff, dist);
            visibility = max(visibility, t);
        }
    }
    
    // 写入结果到输出纹理
    textureStore(output_texture, coords, vec4<f32>(visibility, 0.0, 0.0, 0.0));
}
