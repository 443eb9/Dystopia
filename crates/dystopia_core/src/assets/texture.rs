use bevy::{
    asset::{Assets, Handle},
    math::UVec2,
    prelude::{FromWorld, Resource, World},
    sprite::TextureAtlasLayout,
};

#[derive(Resource)]
pub struct TextureAtlasLayouts {
    pub characters: Handle<TextureAtlasLayout>,
}

impl FromWorld for TextureAtlasLayouts {
    fn from_world(world: &mut World) -> Self {
        let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        Self {
            characters: layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                1,
                1,
                None,
                None,
            )),
        }
    }
}
