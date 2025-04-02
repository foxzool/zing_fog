use crate::fog::{FogOfWarConfig, FogSettings};
use bevy::app::{App, Plugin};
use crate::chunk::ChunkManager;

pub mod prelude;

mod fog;
mod chunk;

pub struct ZingFogPlugins;

impl Plugin for ZingFogPlugins {
    fn build(&self, app: &mut App) {
        app.init_resource::<FogOfWarConfig>()
            .init_resource::<FogSettings>()
            .init_resource::<ChunkManager>()

        ;
    }
}
