use bevy::{
    app::{App, Plugin},
    color::{Color, LinearRgba},
    math::{Vec2, Vec4},
    prelude::{
        Camera, Commands, Component, Entity, GlobalTransform, Query, Res, ResMut, Resource, Shader,
        Time, With,
    },
    reflect::Reflect,
    render::{
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        render_resource::{BufferInitDescriptor, BufferUsages, DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        view::ExtractedView,
    },
};
use bevy::color::ColorToComponents;

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

/// 迷雾设置
/// Fog settings
#[derive(Component, Clone, Reflect, ExtractComponent)]
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
            clear_radius: 0.3,  // 默认相机周围透明区域半径 / Default clear radius
            clear_falloff: 0.1, // 默认边缘过渡宽度 / Default edge falloff width
        }
    }
}

/// 迷雾设置的GPU表示
/// GPU representation of fog settings
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct GpuFogSettings {
    color: LinearRgba,
    center: Vec2, // 迷雾中心位置 / fog center position
    density: f32,
    range: f32,         // 迷雾范围 / fog range
    time: f32,          // 时间（用于动画） / time (for animation)
    clear_radius: f32,  // 相机周围的透明半径 / clear radius around camera
    clear_falloff: f32, // 边缘过渡效果 / edge falloff effect
}

#[derive(Default, Resource)]
pub struct FogOfWarMeta {
    pub gpu_fog_settings: DynamicUniformBuffer<GpuFogSettings>,
}

pub fn prepare_fog_settings(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut fog_meta: ResMut<FogOfWarMeta>,
    views: Query<(Entity, &GlobalTransform, &FogSettings), With<ExtractedView>>,
    time: Res<Time>,
) {
    let views_iter = views.iter();
    let view_count = views_iter.len();
    let Some(mut writer) = fog_meta
        .gpu_fog_settings
        .get_writer(view_count, &render_device, &render_queue)
    else {
        return;
    };
    for (entity, transform, fog_settings) in views_iter {
        let camera_position = transform.translation().truncate();

        // 获取当前时间用于迷雾动画
        // Get current time for fog animation
        let elapsed_time = time.elapsed_secs();

        let settings = GpuFogSettings {
            color: fog_settings.color.to_linear(),
            center: camera_position, // 使用相机位置作为迷雾中心 / Use camera position as fog center
            density: fog_settings.density,
            range: fog_settings.fog_range,
            time: elapsed_time,
            clear_radius: fog_settings.clear_radius, /* 相机周围的透明区域半径 / Clear radius around camera */
            clear_falloff: fog_settings.clear_falloff, // 边缘过渡效果宽度 / Edge falloff width
        };

        commands.entity(entity).insert(ViewFogOfWarUniformOffset {
            offset: writer.write(&settings),
        });
    }

}


#[derive(Component)]
pub struct ViewFogOfWarUniformOffset {
    pub offset: u32,
}