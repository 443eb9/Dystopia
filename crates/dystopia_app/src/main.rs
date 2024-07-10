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
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCamPlugin;
use dystopia_core::DystopiaCorePlugin;

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
                        present_mode: PresentMode::AutoVsync,
                        title: "Dystopia".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            DystopiaCorePlugin,
            PanCamPlugin::default(),
            debug::DystopiaDebugPlugin,
            WorldInspectorPlugin::default(),
        ))
        .run();
}
