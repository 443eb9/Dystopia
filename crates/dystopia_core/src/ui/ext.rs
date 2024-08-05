use bevy::{
    prelude::TextBundle,
    text::{Text, TextStyle},
    ui::Style,
};

/// Create a default UI element with no data but with style.
pub trait DefaultWithStyle<S = Style> {
    fn default_with_style(style: S) -> Self;
}

impl DefaultWithStyle<TextStyle> for TextBundle {
    fn default_with_style(style: TextStyle) -> Self {
        Self::from_section(String::default(), style)
    }
}

impl DefaultWithStyle<TextStyle> for Text {
    fn default_with_style(style: TextStyle) -> Self {
        Self::from_section(String::default(), style)
    }
}
