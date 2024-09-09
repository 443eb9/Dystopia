use bevy::prelude::{Deref, DerefMut};

/// Wrapper for color alpha.
#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct Alpha(f32);

impl Alpha {
    pub fn new(alpha: f32) -> Self {
        Self(alpha)
    }
}
