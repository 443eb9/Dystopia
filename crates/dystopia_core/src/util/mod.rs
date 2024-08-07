use bevy::{
    app::{App, Plugin, Update},
    prelude::{Commands, Component, Entity, Query, Visibility},
};

pub mod macros;

pub struct DystopiaUtilPlugin;

impl Plugin for DystopiaUtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_visibility);
    }
}

#[derive(Component)]
pub struct DeferredVisibilityChange {
    pub frames: u32,
    pub vis: Visibility,
}

fn apply_visibility(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DeferredVisibilityChange, &mut Visibility)>,
) {
    for (entity, mut delay, mut vis) in &mut query {
        if delay.frames == 0 {
            *vis = delay.vis;
            commands.entity(entity).remove::<DeferredVisibilityChange>();
        } else {
            delay.frames -= 1;
        }
    }
}
