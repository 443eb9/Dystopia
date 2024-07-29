use bevy::{
    asset::Handle,
    prelude::{Entity, Image},
    text::Text,
    ui::{UiImage, Val},
};

/// Generate a component for ui data that contains all entities.
pub trait AsBuiltComponent {}

impl AsBuiltComponent for Entity {}

/// Primitive ui data is data that can be spawned into world and be stored
/// using one variable.
pub trait AsBuiltUiElement {
    type BuiltType;
}

#[macro_export]
macro_rules! impl_built_to_entity {
    ($ty: ty) => {
        impl AsBuiltUiElement for $ty {
            type BuiltType = Entity;
        }
    };
}

impl_built_to_entity!(i32);
impl_built_to_entity!(u32);
impl_built_to_entity!(f32);
impl_built_to_entity!(String);
impl_built_to_entity!(Val);

impl<T: AsBuiltUiElement> AsBuiltUiElement for Vec<T> {
    type BuiltType = Vec<Entity>;
}

pub trait AsOriginalUiData {
    type OriginalType;
}

#[macro_export]
macro_rules! impl_as_original_ui_data {
    ($built: ty, $original: ty) => {
        impl AsOriginalUiData for $built {
            type OriginalType = $original;
        }
    };
}

impl_as_original_ui_data!(Text, String);
impl_as_original_ui_data!(UiImage, Handle<Image>);
