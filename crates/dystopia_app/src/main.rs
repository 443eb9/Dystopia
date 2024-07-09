use bevy::{
    app::App,
    prelude::PluginGroup,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use dystopia_core::InfGdnCorePlugin;

mod debug;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        title: "Infinity Garden".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            InfGdnCorePlugin,
            debug::InfGdnDebugPlugin,
        ))
        .run();
}
