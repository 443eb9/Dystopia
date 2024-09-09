use bevy::prelude::{Deref, DerefMut};

use crate::tuple_struct_new;

/// Wrapper for color alpha.
#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct Alpha(f32);
tuple_struct_new!(Alpha, f32);
