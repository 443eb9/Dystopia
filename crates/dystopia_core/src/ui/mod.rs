use bevy::{
    app::{App, Plugin, Update},
    asset::{load_internal_binary_asset, Handle},
    prelude::{ChildBuilder, Entity},
    text::Font,
};

use crate::ui::common::UiAggregate;

pub mod common;
pub mod primitive;
pub mod scrollable_list;
pub mod body_data_panel;

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

        app.add_systems(Update, scrollable_list::handle_scroll);
    }
}

pub trait UiBuilder {
    fn build_ui<U: UiAggregate>(&mut self, elem: U, style: U::Style) -> Entity;
}

impl UiBuilder for ChildBuilder<'_> {
    #[inline]
    fn build_ui<U: UiAggregate>(&mut self, elem: U, style: U::Style) -> Entity {
        elem.build(self, style)
    }
}
