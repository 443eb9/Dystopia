use bevy::{
    app::{App, Plugin, Update},
    asset::{load_internal_binary_asset, Handle},
    prelude::{
        in_state, ChildBuilder, Deref, Entity, FromWorld, IntoSystemConfigs, NodeBundle, Resource,
        World,
    },
    text::{Font, Text},
    ui::{Style, UiImage, Val},
};

use crate::{
    input::RayTransparent, schedule::state::GameState, ui::body_data_panel::BodyDataPanelPlugin,
};

pub mod body_data_panel;
pub mod button;
pub mod ext;
pub mod macros;
pub mod preset;
pub mod primitive;
pub mod scrollable_list;

pub const FUSION_PIXEL: Handle<Font> = Handle::weak_from_u128(789641049865321367040365478967874510);

pub struct DystopiaUiPlugin;

impl Plugin for DystopiaUiPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            FUSION_PIXEL,
            "fusion-pixel-10px-monospaced.otf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );

        app.add_plugins(BodyDataPanelPlugin)
            .add_systems(
                Update,
                (
                    scrollable_list::init_structure,
                    scrollable_list::handle_scroll,
                    button::handle_button_close_click,
                )
                    .run_if(in_state(GameState::Simulate)),
            )
            .add_systems(
                Update,
                (
                    primitive::update_primitive_data::<Text>,
                    primitive::update_primitive_data::<UiImage>,
                )
                    .run_if(in_state(GameState::Simulate)),
            )
            .init_resource::<GlobalUiRoot>();
    }
}

#[derive(Resource, Deref)]
pub struct GlobalUiRoot(Entity);

impl FromWorld for GlobalUiRoot {
    fn from_world(world: &mut World) -> Self {
        Self(
            world
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    RayTransparent,
                ))
                .id(),
        )
    }
}

pub trait UiAggregate {
    type Style;

    /// Build the ui and spawn them into world.
    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity;
}

pub trait UiBuilder {
    fn build_ui<U: UiAggregate>(&mut self, elem: &U, style: U::Style) -> Entity;
}

impl UiBuilder for ChildBuilder<'_> {
    #[inline]
    fn build_ui<U: UiAggregate>(&mut self, elem: &U, style: U::Style) -> Entity {
        elem.build(self, style)
    }
}
