use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    asset::Handle,
    color::{palettes::css::BLACK, Color},
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::{
        BuildChildren, ChildBuilder, Component, Entity, EventReader, GlobalTransform, Label,
        NodeBundle, Parent, Query, TextBundle, With,
    },
    text::{Font, TextStyle},
    ui::{FocusPolicy, JustifyContent, Node, Overflow, Style, Val},
    window::Window,
};
use dystopia_derive::AsBuiltComponent;

use crate::ui::{common::UiAggregate, primitive::PrimitveUiData, FUSION_PIXEL};

#[derive(Component, AsBuiltComponent)]
pub struct ScrollableList {
    pub elements: Vec<ListElement>,
}

pub struct ScrollableListStyle {
    pub list_style: Style,
    pub element_style: ListElementStyle,
}

impl UiAggregate for ScrollableList {
    type Style = ScrollableListStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut elements = Vec::with_capacity(self.elements.len());

        parent
            .spawn(NodeBundle {
                style: Style {
                    overflow: Overflow::clip_y(),
                    ..style.list_style.clone()
                },
                focus_policy: FocusPolicy::Block,
                ..Default::default()
            })
            .with_children(|root| {
                root.spawn((
                    NodeBundle {
                        style: Style {
                            width: Default::default(),
                            height: Default::default(),
                            ..style.list_style
                        },
                        ..Default::default()
                    },
                    ScrollableListInnerContainer::default(),
                ))
                .with_children(|panel| {
                    for elem in &self.elements {
                        elements.push(elem.build(panel, style.element_style.clone()));
                    }
                });
            })
            .insert(BuiltScrollableList { elements })
            .id()
    }
}

#[derive(Component, AsBuiltComponent)]
pub struct ListElement {
    pub title: String,
    pub value: String,
}

impl PrimitveUiData for ListElement {
    type BuiltType = BuiltListElement;
}

#[derive(Clone)]
pub struct ListElementStyle {
    pub font: Handle<Font>,
    pub font_size: f32,
    pub title_color: Color,
    pub value_color: Color,
    pub height: Val,
}

impl Default for ListElementStyle {
    fn default() -> Self {
        Self {
            font: FUSION_PIXEL,
            font_size: 16.,
            title_color: BLACK.into(),
            value_color: BLACK.into(),
            height: Val::Px(20.),
        }
    }
}

impl UiAggregate for ListElement {
    type Style = ListElementStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut title = None;
        let mut value = None;

        parent
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: style.height,
                    min_height: style.height,
                    max_height: style.height,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(|root| {
                // Title
                title = Some(
                    root.spawn((
                        TextBundle::from_section(
                            self.title.clone(),
                            TextStyle {
                                font: style.font.clone(),
                                font_size: style.font_size,
                                color: style.title_color,
                            },
                        ),
                        Label,
                        AccessibilityNode(NodeBuilder::new(Role::ListItem)),
                    ))
                    .id(),
                );

                // Value
                value = Some(
                    root.spawn((
                        TextBundle::from_section(
                            self.value.clone(),
                            TextStyle {
                                font: style.font.clone(),
                                font_size: style.font_size,
                                color: style.value_color,
                            },
                        ),
                        Label,
                    ))
                    .id(),
                );
            })
            .insert(BuiltListElement {
                title: title.unwrap(),
                value: value.unwrap(),
            })
            .id()
    }
}

#[derive(Component, Default)]
pub struct ScrollableListInnerContainer {
    position: f32,
}

pub fn handle_scroll(
    window: Query<&Window>,
    mut mouse_wheel: EventReader<MouseWheel>,
    list_query: Query<(&Node, &GlobalTransform)>,
    mut inner_container_query: Query<
        (
            &mut Style,
            &Node,
            &Parent,
            &mut ScrollableListInnerContainer,
        ),
        With<ScrollableListInnerContainer>,
    >,
) {
    let Some(cursor_pos) = window
        .get_single()
        .map(|w| w.cursor_position())
        .expect("Multiple windows detected, which is not allowed.")
    else {
        return;
    };

    for scroll in mouse_wheel.read() {
        for (mut style, node, parent, mut inner_container) in &mut inner_container_query {
            let (list, list_transform) = list_query.get(parent.get()).unwrap();
            if !list.logical_rect(list_transform).contains(cursor_pos) {
                return;
            }

            let inner_container_height = node.size().y;
            let max_scroll = (inner_container_height - list.size().y).max(0.);

            let dy = match scroll.unit {
                MouseScrollUnit::Line => scroll.y * 20.,
                MouseScrollUnit::Pixel => scroll.y,
            };

            inner_container.position += dy;
            inner_container.position = inner_container.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(inner_container.position);
        }
    }
}
