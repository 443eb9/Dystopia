use bevy::{
    prelude::{Component, Entity},
    ui::Val,
};

/// Generate a component for ui data that contains all entities.
pub trait AsBuiltComponent {}

impl AsBuiltComponent for Entity {}

/// Primitive ui data is data that can be spawned into world and be stored
/// using one variable.
pub trait PrimitveUiData {
    type BuiltType;
}

macro_rules! impl_built_to_entity {
    ($ty: ty) => {
        impl PrimitveUiData for $ty {
            type BuiltType = Entity;
        }
    };
}

impl_built_to_entity!(i32);
impl_built_to_entity!(u32);
impl_built_to_entity!(f32);
impl_built_to_entity!(String);
impl_built_to_entity!(Val);

impl<T: PrimitveUiData> PrimitveUiData for Vec<T> {
    type BuiltType = Vec<Entity>;
}
