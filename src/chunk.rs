use bevy::{prelude::*, render::sync_world::SyncToRenderWorld};
use std::collections::{HashMap, HashSet};
use bevy::render::extract_component::ExtractComponent;
use crate::fog::FogOfWarConfig;

/// 地图区块坐标
/// Map chunk coordinates
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

/// 区块状态
/// Chunk state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum ChunkVisibility {
    /// 未探索 (Unexplored)
    Unexplored,
    /// 已探索但当前不可见 (Explored but not currently visible)
    Explored,
    /// 当前可见 (Currently visible)
    Visible,
}

/// 区块数据
/// Chunk data
#[derive(Component, Reflect)]
pub struct FogChunk {
    pub visibility: ChunkVisibility,
    pub last_visible_time: f32,
}

/// 存储所有激活的区块
/// Stores all active chunks
#[derive(Resource, Default)]
pub struct FogChunkManager {
    /// 激活的区块映射 (Active chunk map)
    pub active_chunks: HashMap<ChunkCoord, Entity>,
    /// 当前可见的区块 (Currently visible chunks)
    pub visible_chunks: HashSet<ChunkCoord>,
    /// 已探索的区块 (Explored chunks)
    pub explored_chunks: HashSet<ChunkCoord>,
}

/// 视野提供者组件
/// Vision provider component
#[derive(Component, Reflect, ExtractComponent, Clone)]
#[require(Transform, Visibility)]
pub struct VisionProvider {
    /// 视野范围（世界单位）
    /// Vision range (world units)
    pub range: f32,
}

/// 更新区块可见性
/// Update chunk visibility
pub fn update_chunk_visibility(
    time: Res<Time>,
    mut commands: Commands,
    config: Res<FogOfWarConfig>,
    mut chunk_manager: ResMut<FogChunkManager>,
    vision_providers: Query<(&GlobalTransform, &VisionProvider)>,
    mut chunks: Query<(Entity, &ChunkCoord, &mut FogChunk)>,
) {
    // 计算当前可见的区块
    // Calculate currently visible chunks
    let mut new_visible_chunks = HashSet::new();
    
    for (transform, vision) in vision_providers.iter() {
        let position = transform.translation().truncate();
        let chunk_range = ((vision.range / config.chunk_size) * 1.5).ceil() as i32;
        
        // 计算视野提供者可见的区块
        // Calculate chunks visible to the vision provider
        for x in -chunk_range..=chunk_range {
            for y in -chunk_range..=chunk_range {
                let center = Vec2::new(position.x, position.y);
                let chunk_pos = ChunkCoord {
                    x: (position.x / config.chunk_size).floor() as i32 + x,
                    y: (position.y / config.chunk_size).floor() as i32 + y,
                };
                
                let chunk_center = Vec2::new(
                    (chunk_pos.x as f32 + 0.5) * config.chunk_size,
                    (chunk_pos.y as f32 + 0.5) * config.chunk_size,
                );
                
                let distance = center.distance(chunk_center);
                if distance <= vision.range {
                    new_visible_chunks.insert(chunk_pos);
                    chunk_manager.explored_chunks.insert(chunk_pos);
                }
            }
        }
    }
    
    // 更新区块可见性状态
    // Update chunk visibility states
    let current_time = time.elapsed_secs();
    
    // 处理不再可见的区块
    // Handle chunks that are no longer visible
    for (entity, coord, mut chunk) in chunks.iter_mut() {
        let is_visible = new_visible_chunks.contains(coord);
        
        match (chunk.visibility, is_visible) {
            (ChunkVisibility::Visible, false) => {
                // 区块从可见变为不可见
                // Chunk transitions from visible to not visible
                chunk.visibility = ChunkVisibility::Explored;
                chunk.last_visible_time = current_time;
            }
            (ChunkVisibility::Unexplored, true) | (ChunkVisibility::Explored, true) => {
                // 区块变为可见
                // Chunk becomes visible
                chunk.visibility = ChunkVisibility::Visible;
            }
            _ => {}
        }
    }
    
    // 更新可见区块集合
    // Update visible chunks collection
    chunk_manager.visible_chunks = new_visible_chunks;
    
    // 克隆集合以避免借用冲突
    // Clone collections to avoid borrowing conflicts
    let visible_chunks = chunk_manager.visible_chunks.clone();
    let explored_chunks = chunk_manager.explored_chunks.clone();
    
    // 创建新的区块实体
    // Create new chunk entities
    for coord in visible_chunks.iter().chain(explored_chunks.iter()) {
        if !chunk_manager.active_chunks.contains_key(coord) {
            // 创建新区块
            // Create new chunk
            let visibility = if visible_chunks.contains(coord) {
                ChunkVisibility::Visible
            } else {
                ChunkVisibility::Explored
            };
            
            let chunk_entity = commands.spawn((
                *coord,
                FogChunk {
                    visibility,
                    last_visible_time: if visibility == ChunkVisibility::Visible {
                        current_time
                    } else {
                        0.0
                    },
                },
            )).id();
            
            chunk_manager.active_chunks.insert(*coord, chunk_entity);
        }
    }
}

/// 加载和卸载区块
/// Load and unload chunks
pub fn manage_chunks(
    time: Res<Time>,
    mut commands: Commands,
    config: Res<FogOfWarConfig>,
    mut chunk_manager: ResMut<FogChunkManager>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    chunks: Query<(Entity, &ChunkCoord, &FogChunk)>,
) {
    // 获取相机位置
    // Get camera position
    let camera_position = if let Ok(transform) = camera_query.get_single() {
        transform.translation().truncate()
    } else {
        return;
    };
    
    // 计算相机所在区块
    // Calculate camera chunk
    let camera_chunk = ChunkCoord {
        x: (camera_position.x / config.chunk_size).floor() as i32,
        y: (camera_position.y / config.chunk_size).floor() as i32,
    };
    
    // 计算加载范围（比视野范围大一些）
    // Calculate loading range (slightly larger than view range)
    let load_range = config.view_range + 2;
    
    // 卸载远离相机的区块
    // Unload chunks far from camera
    let current_time = time.elapsed_secs();
    let mut chunks_to_remove = Vec::new();
    
    for (entity, coord, chunk) in chunks.iter() {
        let dx = (coord.x - camera_chunk.x).abs();
        let dy = (coord.y - camera_chunk.y).abs();
        let distance = (dx * dx + dy * dy) as f32;
        
        // 如果区块太远且不可见，考虑卸载
        // If chunk is too far and not visible, consider unloading
        if distance > (load_range * load_range) as f32 {
            // 如果是已探索区块，只有在一定时间后才卸载
            // If it's an explored chunk, only unload after some time
            if chunk.visibility == ChunkVisibility::Explored {
                let time_since_visible = current_time - chunk.last_visible_time;
                if time_since_visible > 60.0 { // 1分钟后卸载 / Unload after 1 minute
                    chunks_to_remove.push((*coord, entity));
                }
            } else if chunk.visibility == ChunkVisibility::Unexplored {
                // 未探索区块可以立即卸载
                // Unexplored chunks can be unloaded immediately
                chunks_to_remove.push((*coord, entity));
            }
        }
    }
    
    // 执行卸载
    // Perform unloading
    for (coord, entity) in chunks_to_remove {
        commands.entity(entity).despawn();
        chunk_manager.active_chunks.remove(&coord);
        // 保留在已探索集合中，这样我们仍然知道它已被探索
        // Keep in explored set so we still know it was explored
    }
}

/// 准备迷雾渲染数据
/// Prepare fog rendering data
pub fn prepare_fog_data(
    chunk_manager: Res<FogChunkManager>,
    chunks: Query<(&ChunkCoord, &FogChunk)>,
) {
    // 这里可以添加与渲染系统的集成代码
    // Here you can add integration code with the rendering system
    // 例如创建或更新用于渲染的纹理
    // For example, create or update textures for rendering
}

/// 战争迷雾区块系统插件
/// War fog chunk system plugin
pub struct FogChunkPlugin;

impl Plugin for FogChunkPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FogChunkManager>()
            .register_type::<ChunkCoord>()
            .register_type::<ChunkVisibility>()
            .register_type::<FogChunk>()
            // .register_type::<VisionProvider>()
            .add_systems(Update, (
                update_chunk_visibility,
                manage_chunks,
                prepare_fog_data,
            ).chain());
    }
}
