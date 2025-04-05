use crate::chunk::VisionProvider;
use bevy::color::ColorToComponents;
use bevy::image::Image;
use bevy::render::render_resource::AsBindGroup;
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
    /// 噪声纹理
    /// Noise texture
    pub noise_texture: Option<Handle<Image>>,
    /// 噪声强度 (0.0-1.0)
    /// Noise intensity (0.0-1.0)
    pub noise_intensity: f32,
    /// 噪声缩放 (影响噪声纹理的缩放比例)
    /// Noise scale (affects the scaling of the noise texture)
    pub noise_scale: f32,
    /// 噪声速度 (用于动态噪声效果)
    /// Noise speed (for dynamic noise effects)
    pub noise_speed: f32,
}

impl Default for FogMaterial {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0), // 黑色迷雾 / Black fog
            noise_texture: None,
            noise_intensity: 1.0,
            noise_scale: 1.0,
            noise_speed: 0.0,
        }
    }
}

/// 迷雾设置的GPU表示
/// GPU representation of fog settings
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct GpuFogMaterial {
    color: LinearRgba,
    use_noise: u32,       // 是否使用噪声纹理 / Whether to use noise texture
    noise_intensity: f32, // 噪声强度 / Noise intensity
    noise_scale: f32,     // 噪声缩放 / Noise scale
    noise_speed: f32,     // 噪声速度 / Noise speed
    time: f32,            // 当前时间 / Current time (for animated noise)
    vision_range: f32,    // 视野范围 / Vision range
    vision_falloff: f32,  // 视野衰减系数 / Vision falloff coefficient
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
    vision_providers: Query<(&GlobalTransform, &VisionProvider)>,
    time: Res<Time>,
) {
    let views_iter = views.iter();
    let view_count = views_iter.len();
    let Some(mut writer) =
        fog_meta
            .gpu_fog_settings
            .get_writer(view_count, &render_device, &render_queue)
    else {
        return;
    };
    for (entity, _transform, fog_settings) in views_iter {
        // 计算所有视野提供者的平均视野范围
        // Calculate average vision range of all vision providers
        let vision_count = vision_providers.iter().count();
        let avg_vision_range = if vision_count > 0 {
            // 如果有视野提供者，计算平均视野范围
            // If there are vision providers, calculate average vision range
            let total_range: f32 = vision_providers.iter().map(|(_transform, provider)| provider.range).sum();
            total_range / vision_count as f32
        } else {
            // 如果没有视野提供者，设置为0
            // If no vision providers, set to 0
            0.0
        };
        
        // 计算视野范围参数
        // Calculate vision range parameters
        let vision_range = if avg_vision_range > 0.0 {
            // 使用更大的值来增强视野效果
            // Use a larger value to enhance vision effect
            // 这里使用固定值0.8来确保视野范围效果明显
            // Using a fixed value of 0.8 to ensure the vision range effect is noticeable
            0.8
        } else {
            0.0
        };
        
        let settings = GpuFogMaterial {
            color: fog_settings.color.to_linear(),
            use_noise: if fog_settings.noise_texture.is_some() {
                1
            } else {
                0
            },
            noise_intensity: fog_settings.noise_intensity,
            noise_scale: fog_settings.noise_scale,
            noise_speed: fog_settings.noise_speed,
            time: time.elapsed_secs(), // 使用当前时间 / Use current time
            vision_range, // 视野范围参数 / Vision range parameter
            vision_falloff: 0.5, // 设置更小的衰减系数使边缘更柔和 / Set smaller falloff coefficient for softer edges
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
