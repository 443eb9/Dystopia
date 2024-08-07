use bevy::{
    input::ButtonState,
    prelude::{
        BuildChildren, ChildBuilder, Commands, Component, Deref, Entity, MouseButton, NodeBundle,
        Query, TextBundle, Visibility, With,
    },
    text::{JustifyText, Text, TextSection, TextStyle},
    ui::{AlignItems, JustifyContent, Style, UiRect, Val},
};
use dystopia_derive::AsBuiltComponent;

use crate::{
    input::MouseInput,
    ui::{
        preset::{PANEL_BACKGROUND, PANEL_TITLE_FONT_SIZE, PANEL_TITLE_TEXT_COLOR},
        UiAggregate, FUSION_PIXEL,
    },
};

#[derive(Component, Deref)]
pub struct ButtonTarget(Entity);

#[derive(Component, AsBuiltComponent)]
pub struct ButtonClose;

pub struct ButtonCloseStyle {
    pub size: Val,
    pub target: Entity,
}

impl UiAggregate for ButtonClose {
    type Style = ButtonCloseStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        parent
            .spawn((
                NodeBundle {
                    style: Style {
                        width: style.size,
                        height: style.size,
                        border: UiRect::all(Val::Px(2.)),
                        ..Default::default()
                    },
                    border_color: PANEL_BACKGROUND.0.into(),
                    ..Default::default()
                },
                ButtonClose,
                ButtonTarget(style.target),
            ))
            .with_children(|button_root| {
                button_root
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|inner_root| {
                        inner_root.spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection::new(
                                    "X",
                                    TextStyle {
                                        font: FUSION_PIXEL,
                                        font_size: PANEL_TITLE_FONT_SIZE,
                                        color: PANEL_TITLE_TEXT_COLOR,
                                    },
                                )],
                                justify: JustifyText::Right,
                                ..Default::default()
                            },
                            // Due to some calculation errors, the character is not at center actually
                            // so we manually adjust it.
                            style: Style {
                                width: Val::Px(12.6),
                                height: Val::Px(21.3),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    });
            })
            .id()
    }
}

pub fn handle_button_close_click(
    mut commands: Commands,
    buttons_query: Query<(&ButtonTarget, &MouseInput), With<ButtonClose>>,
) {
    for (target, input) in &buttons_query {
        if input.button == MouseButton::Left && input.state == ButtonState::Pressed {
            commands.entity(**target).insert(Visibility::Hidden);
        }
    }
}
