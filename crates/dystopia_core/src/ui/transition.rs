use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    color::LinearRgba,
    prelude::{Animatable, Commands, Component, Entity, Query, Res},
    sprite::Sprite,
    text::Text,
    time::{Real, Time},
};

pub struct MainTransitionablePlugin;

impl Plugin for MainTransitionablePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TransitionableUiPlugin::<Text, (usize, LinearRgba)>::default(),
            TransitionableUiPlugin::<Sprite, LinearRgba>::default(),
        ));
    }
}

pub struct TransitionableUiPlugin<T, P>(PhantomData<(T, P)>)
where
    T: TransitionableUi<P>,
    P: Send + Sync + 'static;

impl<T, P> Default for TransitionableUiPlugin<T, P>
where
    T: TransitionableUi<P>,
    P: Send + Sync + 'static,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, P> Plugin for TransitionableUiPlugin<T, P>
where
    T: TransitionableUi<P>,
    P: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_transitioner::<T, P>);
    }
}

pub trait TransitionableUi<T>: Component {
    fn interpolate(&mut self, target: &T, t: f32);
}

impl TransitionableUi<(usize, LinearRgba)> for Text {
    #[inline]
    fn interpolate(&mut self, target: &(usize, LinearRgba), t: f32) {
        let color = &mut self.sections[target.0].style.color;
        *color = LinearRgba::interpolate(&color.to_linear(), &target.1, t).into();
    }
}

impl TransitionableUi<LinearRgba> for Sprite {
    #[inline]
    fn interpolate(&mut self, target: &LinearRgba, t: f32) {
        self.color = LinearRgba::interpolate(&self.color.to_linear(), target, t).into();
    }
}

#[derive(Component)]
pub struct UiTransition<T, P>
where
    T: TransitionableUi<P>,
    P: Send + Sync + 'static,
{
    to: P,
    duration: f32,
    elapsed: f32,
    _marker: PhantomData<T>,
}

impl<T, P> UiTransition<T, P>
where
    T: TransitionableUi<P>,
    P: Send + Sync + 'static,
{
    pub fn new(to: P, duration: f32) -> Self {
        Self {
            to,
            duration,
            elapsed: 0.,
            _marker: Default::default(),
        }
    }
}

fn ui_transitioner<T, P>(
    mut commands: Commands,
    mut transitions_query: Query<(Entity, &mut UiTransition<T, P>, &mut T)>,
    time: Res<Time<Real>>,
) where
    T: TransitionableUi<P>,
    P: Send + Sync + 'static,
{
    for (entity, mut transition, mut component) in &mut transitions_query {
        transition.elapsed += time.delta_seconds();
        component.interpolate(&transition.to, transition.elapsed / transition.duration);

        if transition.elapsed > transition.duration {
            commands.entity(entity).remove::<UiTransition<T, P>>();
            continue;
        }
    }
}
