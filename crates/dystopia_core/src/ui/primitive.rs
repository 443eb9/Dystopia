use bevy::{
    asset::Handle,
    prelude::{Component, Entity, Image, ParallelCommands, Query},
    text::Text,
    ui::UiImage,
};

/// Generate a component for ui data that contains all entities that
/// have components to display corresponding data.
pub trait AsBuiltComponent {
    const NUM_FIELDS: usize;
}

pub trait AsOriginalUiData {
    type OriginalType: Send + Sync + Clone;
}

pub trait AsUiComponent {
    type UiComponent;
}

#[macro_export]
macro_rules! link_component_and_data {
    ($built: ty, $original: ty) => {
        impl AsOriginalUiData for $built {
            type OriginalType = $original;
        }

        impl AsUiComponent for $original {
            type UiComponent = $built;
        }
    };
}

link_component_and_data!(Text, String);
link_component_and_data!(UiImage, Handle<Image>);

pub trait UpdatableUiComponent: AsOriginalUiData + Component {
    fn update(&mut self, data: Self::OriginalType);
}

impl UpdatableUiComponent for Text {
    fn update(&mut self, data: Self::OriginalType) {
        self.sections[0].value = data;
    }
}

impl UpdatableUiComponent for UiImage {
    fn update(&mut self, data: Self::OriginalType) {
        self.texture = data;
    }
}

#[derive(Component)]
pub struct PrimitiveDataUpdate<T: AsOriginalUiData> {
    pub new: T::OriginalType,
}

pub fn update_primitive_data<T: UpdatableUiComponent>(
    commands: ParallelCommands,
    mut updates_query: Query<(Entity, &PrimitiveDataUpdate<T>, &mut T)>,
) {
    updates_query
        .par_iter_mut()
        .for_each(|(entity, update, mut component)| {
            component.update(update.new.clone());
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<PrimitiveDataUpdate<T>>();
            })
        });
}
