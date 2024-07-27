use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    asset::Handle,
    color::{
        palettes::css::{BLACK, WHITE},
        Color,
    },
    prelude::{BuildChildren, ChildBuilder, Component, Entity, Label, NodeBundle},
    text::{Font, Text, TextStyle},
    ui::{FlexDirection, JustifyContent, Style, Val},
};
use dystopia_derive::AsBuiltComponent;

use crate::ui::{
    primitive::{AsBuiltComponent, PrimitveUiData},
    FUSION_PIXEL,
};

pub trait UiAggregate: Component {
    type Style;

    /// Build the ui and spawn them into world.
    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity;

    // /// Update ui when it's data is changed.
    // fn update(&self, commands: &mut Commands);
}

#[derive(Component, AsBuiltComponent)]
pub struct CommonPanel {
    pub title: String,
}

pub struct CommonPanelStyle {
    pub width: Val,
    pub height: Val,
    pub font: Handle<Font>,
    pub title_size: f32,
}

impl Default for CommonPanelStyle {
    fn default() -> Self {
        Self {
            width: Val::Px(300.),
            height: Val::Px(600.),
            font: FUSION_PIXEL,
            title_size: 32.,
        }
    }
}

// impl UiAggregate for CommonPanel {
//     type BuiltComponent = BuiltCommonPanel;

//     type Style = CommonPanelStyle;

//     fn build(&self, builder: &mut ChildBuilder, style: Self::Style) -> Self::BuiltComponent {
//         let mut title = None;

//         builder
//             .spawn(NodeBundle {
//                 style: Style {
//                     width: style.width,
//                     height: style.height,
//                     flex_direction: FlexDirection::Column,
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             })
//             .with_children(|root| {
//                 // Title
//                 title = Some(
//                     root.spawn(Text::from_section(
//                         &self.title,
//                         TextStyle {
//                             font: style.font.clone(),
//                             font_size: style.title_size,
//                             color: WHITE.into(),
//                         },
//                     ))
//                     .id(),
//                 );
//             });

//         BuiltCommonPanel {
//             title: title.unwrap(),
//         }
//     }
// }
