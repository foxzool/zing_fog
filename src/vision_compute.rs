use crate::prelude::VisionProvider;
use crate::{VISIBILITY_TEXTURE_FORMAT, VISIBILITY_TEXTURE_SIZE};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::lifetimeless::Read;
use bevy::render::render_graph::{RenderLabel, ViewNode};
use bevy::render::render_resource::binding_types::{
    storage_buffer, storage_buffer_read_only, texture_storage_2d, uniform_buffer,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::CachedTexture;
use bevy::render::view::{ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::{Extract, RenderApp};
use bevy::{
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::RenderContext,
    },
};
use bytemuck::Pod;
use bytemuck::Zeroable;

// 视野参数在 GPU 中的表示
#[derive(Debug, Clone, Copy, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct GpuVisionParams {
    position: Vec2,
    range: f32,
    falloff: f32,
}

// 视野参数资源
#[derive(Resource, Default)]
pub struct VisionParamsResource {
    pub params: Vec<GpuVisionParams>,
    pub buffer: Option<Buffer>,
}

// 计算管线
#[derive(Resource)]
pub struct VisionComputePipeline {
    pub pipeline_id: CachedComputePipelineId,
    pub view_bind_group_layout: BindGroupLayout,
    pub data_bind_group_layout: BindGroupLayout,
    texture_descriptor: TextureDescriptor<'static>,
}

impl FromWorld for VisionComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // 创建计算着色器
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/vision_compute.wgsl");

        // 创建视图绑定组布局 (group 0)
        let view_bind_group_layout = render_device.create_bind_group_layout(
            "vision_compute_view_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // view uniform (group 0, binding 0)
                    uniform_buffer::<ViewUniform>(true),
                ),
            ),
        );

        // 创建自定义数据绑定组布局 (group 1)
        let data_bind_group_layout = render_device.create_bind_group_layout(
            "vision_compute_data_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Vision params storage buffer (group 1, binding 0)
                    storage_buffer_read_only::<GpuVisionParams>(false),
                    // Output texture (group 1, binding 1)
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        // 创建计算管线
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("vision_compute_pipeline".into()),
            layout: vec![view_bind_group_layout.clone(), data_bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });

        // 创建纹理描述符
        let texture_descriptor = TextureDescriptor {
            label: Some("visibility_texture"),
            size: Extent3d {
                width: VISIBILITY_TEXTURE_SIZE,
                height: VISIBILITY_TEXTURE_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: VISIBILITY_TEXTURE_FORMAT,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        Self {
            pipeline_id,
            view_bind_group_layout,
            data_bind_group_layout,
            texture_descriptor,
        }
    }
}

// 更新视野参数的 system
pub fn update_vision_params(
    mut vision_params: ResMut<VisionParamsResource>,
    render_device: Res<RenderDevice>,
    query: Extract<Query<(&GlobalTransform, &VisionProvider)>>,
) {
    let params: Vec<GpuVisionParams> = query
        .iter()
        .map(|(transform, provider)| GpuVisionParams {
            position: transform.translation().truncate(),
            range: provider.range,
            falloff: 0.5,
        })
        .collect();

    vision_params.params = params;

    // 更新或创建缓冲区
    if vision_params.params.is_empty() {
        vision_params.buffer = None;
    } else {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("vision_params_buffer"),
            contents: bytemuck::cast_slice(&vision_params.params),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        vision_params.buffer = Some(buffer);
    }
}

// 可见性纹理资源
/// Visibility texture resource
#[derive(Resource, Default)]
pub struct VisibilityTextureResource {
    pub texture: Option<CachedTexture>,
}

// 视野计算插件
pub struct VisionComputePlugin;

impl Plugin for VisionComputePlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<VisionParamsResource>()
            .init_resource::<VisibilityTextureResource>()
            .add_systems(ExtractSchedule, update_vision_params);
    }
}

#[derive(RenderLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct VisionComputeLabel;

// 计算节点
pub struct VisionComputeNode {
    pub visibility_texture: Option<CachedTexture>,
    vision_params_buffer: Option<Buffer>,
    result_buffer: Option<Buffer>,
}

impl Default for VisionComputeNode {
    fn default() -> Self {
        Self {
            visibility_texture: None,
            vision_params_buffer: None,
            result_buffer: None,
        }
    }
}

impl ViewNode for VisionComputeNode {
    type ViewQuery = (Read<ViewUniformOffset>);

    fn update(&mut self, world: &mut World) {
        // 首先获取所有需要的资源
        let pipeline = world.resource::<VisionComputePipeline>();
        let vision_params = world.resource::<VisionParamsResource>();
        let render_device = world.resource::<RenderDevice>();

        // 使用已经准备好的缓冲区
        self.vision_params_buffer = vision_params.buffer.clone();

        // 创建结果缓冲区
        if self.result_buffer.is_none() {
            self.result_buffer = Some(render_device.create_buffer(&BufferDescriptor {
                label: Some("vision_compute_result_buffer"),
                size: 4, // 一个 f32 的大小
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }));
        }

        // 确保可见性纹理存在
        if self.visibility_texture.is_none() {
            let texture = render_device.create_texture(&pipeline.texture_descriptor);
            let default_view = texture.create_view(&TextureViewDescriptor::default());
            self.visibility_texture = Some(CachedTexture {
                texture,
                default_view,
            });
        }

        // 更新可见性纹理资源
        // Update visibility texture resource
        if let Some(visibility_texture) = &self.visibility_texture {
            let mut visibility_resource = world.resource_mut::<VisibilityTextureResource>();
            visibility_resource.texture = Some(visibility_texture.clone());
        }
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_uniform_offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<VisionComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let view_uniforms = world.resource::<ViewUniforms>();

        let Some(view_uniforms_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let Some(vision_params_buffer) = &self.vision_params_buffer else {
            return Ok(());
        };

        let visibility_texture = self.visibility_texture.as_ref().unwrap();

        // 创建视图绑定组 (group 0)
        let view_bind_group = render_context.render_device().create_bind_group(
            Some("vision_compute_view_bind_group"),
            &pipeline.view_bind_group_layout,
            &BindGroupEntries::sequential((
                view_uniforms_binding,
            )),
        );
        
        // 创建数据绑定组 (group 1)
        let data_bind_group = render_context.render_device().create_bind_group(
            Some("vision_compute_data_bind_group"),
            &pipeline.data_bind_group_layout,
            &BindGroupEntries::sequential((
                vision_params_buffer.as_entire_binding(),
                &visibility_texture.default_view,
            )),
        );

        // 分派计算着色器
        let mut compute_pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, &view_bind_group, &[view_uniform_offset.offset]);
        compute_pass.set_bind_group(1, &data_bind_group, &[]);

        let workgroup_size = 8;
        let dispatch_size = (VISIBILITY_TEXTURE_SIZE + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);

        Ok(())
    }
}
