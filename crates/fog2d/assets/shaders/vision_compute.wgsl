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

@group(0) @binding(0) var<storage, read> visions: VisionArray;
@group(0) @binding(1) var<storage, read_write> result: ComputeResult;
@group(0) @binding(2) var output_texture: texture_storage_2d<r32float, write>;

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
        let dist = distance(uv, vision.position);
        
        // 计算该视野提供者的可见性贡献
        let normalized_dist = dist / vision.range;
        let visibility = 1.0 - smoothstep(0.6, 1.0, normalized_dist);
        
        // 取最大可见性
        max_visibility = max(max_visibility, visibility);
    }
    
    // 存储结果到纹理
    textureStore(output_texture, global_id.xy, vec4<f32>(max_visibility));
}
