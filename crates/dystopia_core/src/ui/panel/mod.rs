use std::marker::PhantomData;

use bevy::prelude::{Deref, Entity, Event, Resource};

pub mod body_data;
pub mod scene_title;
pub mod system_statistics;

#[derive(Event, Deref)]
pub struct PanelTargetChange<P: Resource, D = Entity> {
    #[deref]
    target: Option<D>,
    _panel: PhantomData<P>,
}

impl<P: Resource, D> PanelTargetChange<P, D> {
    pub fn some(target: D) -> Self {
        Self {
            target: Some(target),
            _panel: Default::default(),
        }
    }

    pub fn none() -> Self {
        Self {
            target: None,
            _panel: Default::default(),
        }
    }
}
