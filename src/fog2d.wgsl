#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput



@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // 直接返回一个纯红色，用于测试着色器是否正常工作
    // Return a pure red color to test if the shader is working properly
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}