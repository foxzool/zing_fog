#import bevy_render::view::View
#import bevy_pbr::view_transformations::{
                          depth_ndc_to_view_z,
                          frag_coord_to_ndc,
                          ndc_to_frag_coord,
                          ndc_to_uv,
                          position_view_to_ndc,
                          position_world_to_ndc,
                          position_world_to_view,
                      },
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


// 使用group(1)避免与Bevy内置绑定冲突
@group(1) @binding(0) var<storage, read> visions: VisionArray;
@group(1) @binding(1) var output_texture: texture_storage_2d<r32float, write>;
// View 应该保持在group(0)，因为它是Bevy的内置绑定
@group(0) @binding(0) var<uniform> view: View;

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
        
        // 1. 将vision的2D世界坐标转换为3D世界坐标 (z=0平面)
        // Convert vision 2D world position to 3D world position (z=0 plane)
        let world_pos = vec3<f32>(vision.position.x, vision.position.y, 0.0);
        
        // 2. 将世界坐标转换为NDC坐标
        // Convert world position to NDC space
        let ndc = position_world_to_ndc(world_pos);
        
        // 3. NDC坐标转换为UV坐标 (0-1范围)
        // Convert NDC to UV coordinates (0-1 range)
        let vision_uv = ndc_to_uv(ndc.xy);
        
        // 计算当前像素与视野中心的距离
        let dist = distance(uv, vision_uv);
        
        // 计算世界空间中的距离比例
        // 首先将当前像素的UV坐标转换回NDC
        let pixel_ndc = frag_coord_to_ndc(vec4<f32>(vec2<f32>(global_id.xy) + vec2<f32>(0.5), 0.0, 1.0));
        
        // 获取视野范围在屏幕空间的比例
        // 创建一个位于视野边缘的点（世界空间）
        let range_point = vec3<f32>(vision.position.x + vision.range, vision.position.y, 0.0);
        // 将其转换为NDC
        let range_ndc = position_world_to_ndc(range_point);
        // 获取中心点和边缘点之间的NDC距离
        let ndc_range = distance(ndc.xy, range_ndc.xy);
        
        // 归一化距离
        let normalized_dist = dist / ndc_range;
        
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
