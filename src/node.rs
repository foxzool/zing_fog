use crate::fog::{FogOfWarMeta, FogSettings, GpuFogSettings, ViewFogOfWarUniformOffset};
use bevy::{
    asset::AssetServer,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::{Commands, Component, Entity, FromWorld, Res, Resource, Shader, With, World},
    render::{
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            BindGroupEntries, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntries,
            BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendFactor,
            BlendOperation, BlendState, BufferBinding, BufferBindingType, BufferInitDescriptor,
            BufferSize, BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            Extent3d, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations,
            PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            ShaderStages, StorageTextureAccess, StoreOp, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, TextureViewDescriptor, VertexState,
            binding_types::{
                storage_buffer_read_only_sized, texture_storage_2d_array, uniform_buffer,
            },
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::{ExtractedView, ViewTarget, ViewUniform, ViewUniforms},
    },
    utils::default,
};
use crate::FOG_2D_SHADER_HANDLE;

/// 迷雾节点名称
/// Fog node name
#[derive(RenderLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FogNode2dLabel;

#[derive(Resource)]
pub struct FogOfWar2dPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for FogOfWar2dPipeline {
    fn from_world(world: &mut World) -> Self {
        // let chunks = world
        //     .query::<&ChunkRingBuffer>()
        //     .iter(&world)
        //     .collect::<Vec<_>>();
        // let views_chunk_count = chunks.iter().map(|c| c.visible()).filter(|b| *b).count() as u32;
        //
        // let settings = world.resource::<FogSettings>();
        // // let chunk_size = settings.chunk_size;


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

        let explored_texture = texture.create_view(&TextureViewDescriptor {
            dimension: Some(bevy::render::render_resource::TextureViewDimension::D2Array),
            ..TextureViewDescriptor::default()
        });

        let bind_group_layout = render_device.create_bind_group_layout(
            "fog_of_war_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<GpuFogSettings>(true),
                    // storage_buffer_read_only_sized(false, None),
                    // texture_storage_2d_array(
                    //     TextureFormat::R8Unorm,
                    //     StorageTextureAccess::ReadWrite,
                    // ),
                    // storage_buffer_read_only_sized(false, None),
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
            // explored_texture: Some(explored_texture),
            // texture: Some(texture),
        }
    }
}

#[derive(Default)]
pub struct FogNode2d;

impl ViewNode for FogNode2d {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<ViewFogOfWarUniformOffset>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target,  view_fog_offset): QueryItem<Self::ViewQuery>,
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

        let bind_group = render_context.render_device().create_bind_group(
            None,
            &fog_of_war_pipeline.bind_group_layout,
            &BindGroupEntries::sequential((
                // view_uniforms_binding,
                settings_binding.clone(),
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

pub fn prepare_bind_groups(mut commands: Commands) {}
