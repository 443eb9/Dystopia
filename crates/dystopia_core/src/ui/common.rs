use bevy::{
    asset::Handle,
    prelude::{ChildBuilder, Component, Entity},
    text::Font,
    ui::Val,
};
use dystopia_derive::AsBuiltComponent;

use crate::ui::FUSION_PIXEL;

pub trait UiAggregate {
    type Style;

    /// Build the ui and spawn them into world.
    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity;

    // /// Update ui when it's data is changed.
    // fn update(&self, commands: &mut Commands);
}
