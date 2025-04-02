use bevy::color::Color;
use bevy::prelude::{Component, Resource};
use bevy::render::extract_component::ExtractComponent;
use bevy::render::extract_resource::ExtractResource;

/// 迷雾相机标记组件
/// Fog camera marker component
#[derive(Component, ExtractComponent, Default, Clone, Copy, Debug)]
pub struct FogCameraMarker;


/// 迷雾战争插件配置
/// Fog of War plugin configuration
#[derive(Resource)]
pub struct FogOfWarConfig {
    /// 区块大小（世界单位）
    /// Chunk size (world units)
    pub chunk_size: f32,
    /// 视野范围（以区块为单位）
    /// View range (in chunks)
    pub view_range: u32,
    /// 是否启用调试绘制
    /// Whether to enable debug drawing
    pub debug_draw: bool,
}

impl Default for FogOfWarConfig {
    fn default() -> Self {
        Self {
            chunk_size: 256.0,
            view_range: 3,
            debug_draw: true,
        }
    }
}


/// 迷雾设置资源
/// Fog settings resource
#[derive(Resource, Clone, ExtractResource)]
pub struct FogSettings {
    /// 迷雾颜色
    /// Fog color
    pub color: Color,
    /// 迷雾密度
    /// Fog density
    pub density: f32,
    /// 迷雾范围
    /// Fog range
    pub fog_range: f32,
    /// 迷雾最大强度
    /// Maximum fog intensity
    pub max_intensity: f32,
    /// 相机周围的透明区域半径
    /// Clear radius around camera
    pub clear_radius: f32,
    /// 边缘过渡效果宽度
    /// Edge falloff width
    pub clear_falloff: f32,
}

impl Default for FogSettings {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0), // 黑色迷雾 / Black fog
            density: 0.05,
            fog_range: 1000.0,
            max_intensity: 0.8,
            clear_radius: 0.3,      // 默认相机周围透明区域半径 / Default clear radius
            clear_falloff: 0.1,      // 默认边缘过渡宽度 / Default edge falloff width
        }
    }
}