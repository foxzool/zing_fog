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
use bevy::image::Image;
use bevy::render::render_resource::AsBindGroup;
use bevy_asset::Handle;

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
pub struct FogMaterial {
    /// 迷雾颜色
    /// Fog color
    pub color: Color,
}

impl Default for FogMaterial {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0), // 黑色迷雾 / Black fog
        }
    }
}

/// 迷雾设置的GPU表示
/// GPU representation of fog settings
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct GpuFogMaterial {
    color: LinearRgba,
}

#[derive(Default, Resource)]
pub struct FogOfWarMeta {
    pub gpu_fog_settings: DynamicUniformBuffer<GpuFogMaterial>,
}

pub fn prepare_fog_settings(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut fog_meta: ResMut<FogOfWarMeta>,
    views: Query<(Entity, &GlobalTransform, &FogMaterial), With<ExtractedView>>,
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


        let settings = GpuFogMaterial {
            color: fog_settings.color.to_linear(),
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