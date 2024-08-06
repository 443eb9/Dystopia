use bevy::app::{App, Plugin};

use crate::{assets::app_ext::DystopiaAssetAppExt, input::mapping::RawInputMappingConfig};

pub mod camera;
pub mod ext;
pub mod mapping;

pub struct DystopiaInputPlugin;

impl Plugin for DystopiaInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_config::<RawInputMappingConfig>();
    }
}
