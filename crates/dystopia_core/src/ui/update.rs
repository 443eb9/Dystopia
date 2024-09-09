use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    asset::Handle,
    color::LinearRgba,
    prelude::{Component, Entity, Image, ParallelCommands, Query},
    text::Text,
    ui::UiImage,
};

use crate::util::alpha::Alpha;

pub struct MainUpdatablePlugin;

impl Plugin for MainUpdatablePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UpdatablePlugin::<Text, String>::default(),
            UpdatablePlugin::<Text, (usize, String)>::default(),
            UpdatablePlugin::<Text, LinearRgba>::default(),
            UpdatablePlugin::<Text, (usize, LinearRgba)>::default(),
            UpdatablePlugin::<Text, Alpha>::default(),
            UpdatablePlugin::<UiImage, Handle<Image>>::default(),
        ));
    }
}

pub struct UpdatablePlugin<U, P>(PhantomData<(U, P)>);

impl<U, P> Default for UpdatablePlugin<U, P> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<U, P> Plugin for UpdatablePlugin<U, P>
where
    U: DataUpdatableUi<P>,
    P: Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_data_updater::<U, P>);
    }
}

/// Get the original type of data. Eg. for [`String`]s, the original component
/// will be [`Text`].
///
/// This is only used in the derive macro of [`AsBuiltComponent`].
pub trait AsOriginalComponent {
    type OriginalComponent;
}

/// Get the updatable data form of that data. For most types, this will be the
/// original type. But for [`LocalizableDataWrapper`](crate::localization::LocalizableDataWrapper)s,
/// this will be [`String`].
///
/// This is only used in the derive macro of [`AsBuiltComponent`].
pub trait AsUpdatableData {
    type UpdatableData;
}

impl AsUpdatableData for String {
    type UpdatableData = Self;
}

impl AsUpdatableData for Handle<Image> {
    type UpdatableData = Self;
}

impl AsUpdatableData for LinearRgba {
    type UpdatableData = Self;
}

pub trait DataUpdatableUi<P>: Component {
    fn update(&mut self, data: P);
}

impl DataUpdatableUi<String> for Text {
    fn update(&mut self, data: String) {
        self.sections[0].value = data;
    }
}

impl AsOriginalComponent for String {
    type OriginalComponent = Text;
}

impl DataUpdatableUi<(usize, String)> for Text {
    fn update(&mut self, data: (usize, String)) {
        self.sections[data.0].value = data.1;
    }
}

impl AsOriginalComponent for (usize, String) {
    type OriginalComponent = Text;
}

impl DataUpdatableUi<(usize, LinearRgba)> for Text {
    fn update(&mut self, data: (usize, LinearRgba)) {
        self.sections[data.0].style.color = data.1.into();
    }
}

impl AsOriginalComponent for (usize, LinearRgba) {
    type OriginalComponent = Text;
}

impl DataUpdatableUi<LinearRgba> for Text {
    fn update(&mut self, data: LinearRgba) {
        for section in &mut self.sections {
            section.style.color = data.into();
        }
    }
}

impl AsOriginalComponent for LinearRgba {
    type OriginalComponent = Text;
}

impl DataUpdatableUi<Handle<Image>> for UiImage {
    fn update(&mut self, data: Handle<Image>) {
        self.texture = data;
    }
}

impl AsOriginalComponent for Handle<Image> {
    type OriginalComponent = UiImage;
}

impl DataUpdatableUi<Alpha> for Text {
    fn update(&mut self, data: Alpha) {
        for section in &mut self.sections {
            bevy::prelude::Alpha::set_alpha(&mut section.style.color, *data);
        }
    }
}

#[derive(Component)]
pub struct UiDataUpdate<U, P>
where
    U: DataUpdatableUi<P>,
    P: Clone + Send + Sync + 'static,
{
    new: P,
    _marker: PhantomData<U>,
}

impl<U, P> UiDataUpdate<U, P>
where
    U: DataUpdatableUi<P>,
    P: Clone + Send + Sync + 'static,
{
    pub fn new(new: P) -> Self {
        Self {
            new,
            _marker: Default::default(),
        }
    }
}

fn ui_data_updater<U, P>(
    commands: ParallelCommands,
    mut updates_query: Query<(Entity, &UiDataUpdate<U, P>, &mut U)>,
) where
    U: DataUpdatableUi<P>,
    P: Clone + Send + Sync + 'static,
{
    updates_query
        .par_iter_mut()
        .for_each(|(entity, update, mut component)| {
            component.update(update.new.clone());
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<UiDataUpdate<U, P>>();
            })
        });
}

/// Generate a component for ui data that contains all entities that
/// have components to display corresponding data.
pub trait AsBuiltComponent {
    const NUM_FIELDS: usize;
}
