#import bevy_render::view::View

// 视野参数结构体
struct VisionParams {
    position: vec2<f32>,  // 世界空间位置
    range: f32,           // 视野范围
    falloff: f32,         // 边缘衰减
};

// 视野参数数组
struct VisionArray {
    data: array<VisionParams>,
};

// 视野计算结果
struct ComputeResult {
    visibility: array<f32>,
};

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> visions: VisionArray;
@group(0) @binding(2) var<storage, read_write> result: ComputeResult;
@group(0) @binding(3) var output_texture: texture_storage_2d<r32float, write>;

// 计算着色器入口点
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(output_texture);
    let uv = vec2<f32>(global_id.xy) / vec2<f32>(dims);
    
    // 计算该像素的可见性
    var max_visibility = 0.0;
    
    // 遍历所有视野提供者
    for (var i = 0u; i < arrayLength(&visions.data); i++) {
        let vision = visions.data[i];
        
        // 将世界坐标转换为纹理坐标 (0-1 范围)
        // Convert world coordinates to texture coordinates (0-1 range)
        // 这里假设世界坐标范围是 [-500, 500]，需要根据实际情况调整
        let vision_uv = (vision.position + vec2<f32>(500.0, 500.0)) / vec2<f32>(1000.0, 1000.0);
        
        let dist = distance(uv, vision_uv);
        
        // 调整视野范围以适应纹理坐标系统
        // Adjust vision range to fit texture coordinate system
        let adjusted_range = vision.range / 1000.0;
        
        // 计算该视野提供者的可见性贡献
        let normalized_dist = dist / adjusted_range;
        // 使用更平滑的衰减曲线
        // Use a smoother falloff curve
        let visibility = 1.0 - smoothstep(0.0, 1.0, normalized_dist);
        
        // 取最大可见性
        max_visibility = max(max_visibility, visibility);
    }
    
    // 确保可见性值在有效范围内并增强对比度
    // Ensure visibility value is in valid range and enhance contrast
    max_visibility = clamp(max_visibility, 0.0, 1.0);
    
    // 存储结果到纹理
    textureStore(output_texture, global_id.xy, vec4<f32>(max_visibility));
}
