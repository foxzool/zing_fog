#import bevy_render::view::View
#import bevy_pbr::view_transformations::{
                          depth_ndc_to_view_z,
                          frag_coord_to_ndc,
                          ndc_to_frag_coord,
                          uv_to_ndc,
                          position_ndc_to_world,
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
@group(0) @binding(0) var<uniform> view: View;

// 计算着色器入口点
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(output_texture);
    let uv = vec2<f32>(global_id.xy) / vec2<f32>(dims);
    let ndc = uv_to_ndc(uv); // uv_to_ndc返回vec2<f32>
    // 创建一个完整的vec3<f32>作为NDC坐标 (x,y来自uv转换，z设为0.0)
    let ndc_pos = vec3<f32>(ndc, 0.0);
    let world_position = position_ndc_to_world(ndc_pos);
    
    // 计算该像素的可见性
    var max_visibility = 0.0;
    
    // 遍历所有视野提供者
    for (var i = 0u; i < arrayLength(&visions.data); i++) {
       let vision = visions.data[i];
       let dist = distance(world_position.xy, vision.position);
       if (dist < vision.range) {
           max_visibility = max(max_visibility, 1.0 - smoothstep(vision.range * vision.falloff, vision.range, dist));
       }
    }
    
    // 确保可见性值在有效范围内并增强对比度
    // Ensure visibility value is in valid range and enhance contrast
    max_visibility = clamp(max_visibility, 0.0, 1.0);
    
    // 存储结果到纹理
    textureStore(output_texture, global_id.xy, vec4<f32>(max_visibility));
}
