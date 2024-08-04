use bevy::prelude::Entity;
use dystopia_derive::LocalizableEnum;

use crate::{gen_localizable_enum, ui::primitive::AsBuiltUiElement};

gen_localizable_enum!(LUiPanel, BodyData);
