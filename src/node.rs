use crate::FOG_2D_SHADER_HANDLE;
use crate::fog::{FogOfWarMeta, GpuFogMaterial, ViewFogOfWarUniformOffset};
use bevy::{
    asset::{AssetServer, Handle},
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::{Commands, FromWorld, Image, Resource, World},
    render::{
        render_asset::RenderAssets,
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BindGroupLayoutEntry,
            BindingResource, BindingType, BlendComponent, BlendState, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, Extent3d, FragmentState, FrontFace, LoadOp,
            MultisampleState, Operations, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, ShaderStages, StoreOp, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, binding_types::uniform_buffer,
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        view::{ViewTarget, ViewUniforms},
    },
    utils::default,
};

/// 迷雾节点名称
/// Fog node name
#[derive(RenderLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FogNode2dLabel;

#[derive(Resource)]
pub struct FogOfWar2dPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
    pub noise_texture: Option<Handle<Image>>,
}

impl FromWorld for FogOfWar2dPipeline {
    fn from_world(world: &mut World) -> Self {
        // 加载噪声纹理
        // Load noise texture
        let asset_server = world.resource::<AssetServer>();
        let noise_texture = asset_server.load("textures/noise.png");

        let render_device = world.resource_mut::<RenderDevice>();

        let texture = render_device.create_texture(&TextureDescriptor {
            label: Some("fog_explored_texture"),
            size: Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 20,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let bind_group_layout = render_device.create_bind_group_layout(
            "fog_of_war_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<GpuFogMaterial>(true),
                    // 添加噪声纹理绑定
                    // Add noise texture binding
                    BindGroupLayoutEntry {
                        ty: BindingType::Texture {
                            sample_type: bevy::render::render_resource::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: bevy::render::render_resource::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                    },
                    // 添加采样器绑定
                    // Add sampler binding
                    BindGroupLayoutEntry {
                        ty: BindingType::Sampler(
                            bevy::render::render_resource::SamplerBindingType::Filtering,
                        ),
                        count: None,
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                    },
                ),
            ),
        );

        let pipeline_id = world.resource_mut::<PipelineCache>().queue_render_pipeline(
            RenderPipelineDescriptor {
                label: Some("fog_of_war_2d_pipeline".into()),
                layout: vec![bind_group_layout.clone()],
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader: FOG_2D_SHADER_HANDLE,
                    shader_defs: vec![],
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb, // 明确指定格式
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: bevy::render::render_resource::BlendFactor::SrcAlpha,
                                dst_factor:
                                    bevy::render::render_resource::BlendFactor::OneMinusSrcAlpha,
                                operation: bevy::render::render_resource::BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: bevy::render::render_resource::BlendFactor::SrcAlpha,
                                dst_factor:
                                    bevy::render::render_resource::BlendFactor::OneMinusSrcAlpha,
                                operation: bevy::render::render_resource::BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            },
        );

        Self {
            bind_group_layout,
            pipeline_id,
            noise_texture: Some(noise_texture),
            // explored_texture: Some(explored_texture),
            // texture: Some(texture),
        }
    }
}

#[derive(Default)]
pub struct FogNode2d;

impl ViewNode for FogNode2d {
    type ViewQuery = (Read<ViewTarget>, Read<ViewFogOfWarUniformOffset>);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, view_fog_offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let fog_of_war_pipeline = world.resource::<FogOfWar2dPipeline>();
        let view_uniforms = world.resource::<ViewUniforms>();

        let Some(view_uniforms_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let Some(pipeline) = pipeline_cache.get_render_pipeline(fog_of_war_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let Some(settings_binding) = world.resource::<FogOfWarMeta>().gpu_fog_settings.binding()
        else {
            return Ok(());
        };

        let view = view_target.main_texture_view();

        // 获取噪声纹理和采样器
        // Get noise texture and sampler
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let noise_texture_id = fog_of_war_pipeline.noise_texture.as_ref().unwrap();
        let fallback_image = world.resource::<bevy::render::texture::FallbackImage>();

        // 获取噪声纹理或使用回退图像
        // Get noise texture or use fallback image
        let noise_texture_view = if let Some(gpu_image) = gpu_images.get(noise_texture_id) {
            &gpu_image.texture_view
        } else {
            &fallback_image.d2.texture_view
        };

        let sampler = &world
            .resource::<bevy::render::texture::FallbackImage>()
            .d2
            .sampler;

        let bind_group = render_context.render_device().create_bind_group(
            None,
            &fog_of_war_pipeline.bind_group_layout,
            &BindGroupEntries::sequential((
                // view_uniforms_binding,
                settings_binding.clone(),
                // 添加噪声纹理和采样器绑定
                // Add noise texture and sampler bindings
                BindingResource::TextureView(noise_texture_view),
                BindingResource::Sampler(sampler),
                // fog_sight_buffers.buffers.into_binding(),
                // fog_of_war_pipeline.explored_texture.as_ref().unwrap(),
                // ring_buffers.buffers.into_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("fog_of_war_2d_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            ..default()
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[view_fog_offset.offset]);

        render_pass.draw(0..3, 0..1);
        Ok(())
    }
}

pub fn prepare_bind_groups(commands: Commands) {}
