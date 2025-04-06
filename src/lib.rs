use crate::chunk::VisionProvider;
use crate::vision_compute::VisionComputePlugin;
use crate::{
    chunk::FogChunkPlugin,
    fog::{FogMaterial, FogOfWarConfig, FogOfWarMeta, prepare_fog_settings},
    node::{FogNode2d, FogNode2dLabel, FogOfWar2dPipeline},
    vision_compute::{VisionComputeNode, VisionComputePipeline},
};
use bevy::prelude::IntoSystemConfigs;
use bevy::render::render_resource::TextureFormat;
use bevy::render::sync_component::SyncComponentPlugin;
use bevy::{
    app::{App, Plugin},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::Shader,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::ExtractComponentPlugin,
        extract_resource::ExtractResourcePlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
    },
};
use bevy_asset::{Handle, load_internal_asset};
use vision_compute::VisionComputeLabel;

pub mod prelude;

mod fog;

mod node;

mod chunk;

mod vision_compute;

#[cfg(feature = "2d")]
pub const FOG_2D_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2645352199453808407);
pub const VISION_COMPUTE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(2645352199453808408);

pub const VISIBILITY_TEXTURE_FORMAT: TextureFormat = TextureFormat::R32Float;
pub const VISIBILITY_TEXTURE_SIZE: u32 = 1024;

pub struct ZingFogPlugins;

impl Plugin for ZingFogPlugins {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "2d")]
        load_internal_asset!(app, FOG_2D_SHADER_HANDLE, "fog2d.wgsl", Shader::from_wgsl);

        app.init_resource::<FogOfWarConfig>();

        app.register_type::<FogMaterial>()
            .add_plugins(ExtractComponentPlugin::<FogMaterial>::default())
            .add_plugins(ExtractComponentPlugin::<VisionProvider>::default())
            .add_plugins(FogChunkPlugin)
            .add_plugins(VisionComputePlugin);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // 将迷雾节点放在 MainTransparentPass 和 EndMainPass 之间
        // Place fog node between MainTransparentPass and EndMainPass
        render_app
            .init_resource::<FogOfWarMeta>()
            .add_systems(
                Render,
                prepare_fog_settings.in_set(RenderSet::PrepareResources),
            )
            .add_render_graph_node::<ViewNodeRunner<FogNode2d>>(Core2d, FogNode2dLabel)
            .add_render_graph_node::<ViewNodeRunner<VisionComputeNode>>(Core2d, VisionComputeLabel)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainTransparentPass,
                    VisionComputeLabel,
                    FogNode2dLabel,
                    Node2d::EndMainPass,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<FogOfWar2dPipeline>();
        render_app.init_resource::<VisionComputePipeline>();
    }
}
