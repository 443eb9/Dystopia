use std::marker::PhantomData;

use bevy::prelude::{Deref, Entity, Event};

pub mod body_data;

#[derive(Event, Deref)]
pub struct PanelTargetChange<P> {
    #[deref]
    target: Option<Entity>,
    _marker: PhantomData<P>,
}

impl<P> PanelTargetChange<P> {
    pub fn some(target: Entity) -> Self {
        Self {
            target: Some(target),
            _marker: Default::default(),
        }
    }

    pub fn none() -> Self {
        Self {
            target: None,
            _marker: Default::default(),
        }
    }
}
