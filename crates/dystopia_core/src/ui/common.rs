use bevy::prelude::{ChildBuilder, Entity};

pub trait UiAggregate {
    type Style;

    /// Build the ui and spawn them into world.
    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity;
}
