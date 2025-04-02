use bevy::app::{App, Plugin};
use crate::fog::FogOfWarConfig;

pub mod prelude;

mod fog;

pub struct ZingFogPlugins;

impl Plugin for ZingFogPlugins {
    fn build(&self, app: &mut App) {
        app.init_resource::<FogOfWarConfig>();
    }
}

