use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::{
        Added, BuildChildren, Children, Commands, Component, Entity, EventReader, NodeBundle,
        Parent, Query, ViewVisibility, With,
    },
    ui::{Node, Overflow, Style, Val},
};

use crate::input::MouseHovering;

/// Mark a container as a scrollable list.
///
/// A scrollable list should have a following structure:
///
/// - Root [`ScrollableList`]
///   - InnerContainer [`ScrollableListInnerContainer`]
///     - Elements
///
/// This structure will be created by system [``], as well as the overflow settings.
/// What you need to do is to create following structure:
///
/// - Root [`ScrollableList`]
///   - Elements
#[derive(Component)]
pub struct ScrollableList;

#[derive(Component, Default)]
pub struct ScrollableListInnerContainer {
    position: f32,
}

pub fn init_structure(
    mut commands: Commands,
    mut list_query: Query<(Entity, &mut Style, &Children), Added<ScrollableList>>,
) {
    for (list_root, mut style, children) in &mut list_query {
        style.overflow = Overflow::clip_y();

        let inner_container = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Default::default(),
                        height: Default::default(),
                        ..style.clone()
                    },
                    ..Default::default()
                },
                ScrollableListInnerContainer::default(),
            ))
            .id();

        commands.entity(inner_container).set_parent(list_root);
        for child in children {
            commands.entity(*child).set_parent(inner_container);
        }
    }
}

pub fn handle_scroll(
    mut mouse_wheel: EventReader<MouseWheel>,
    node_query: Query<&Node>,
    mut inner_container_query: Query<
        (
            &mut Style,
            &Node,
            &Parent,
            &mut ScrollableListInnerContainer,
            &ViewVisibility,
        ),
        With<MouseHovering>,
    >,
) {
    for scroll in mouse_wheel.read() {
        inner_container_query.par_iter_mut().for_each(
            |(mut style, node, parent, mut inner_container, vis)| {
                if !vis.get() {
                    return;
                }

                let list = node_query.get(parent.get()).unwrap();

                let inner_container_height = node.size().y;
                let max_scroll = (inner_container_height - list.size().y).max(0.);

                let dy = match scroll.unit {
                    MouseScrollUnit::Line => scroll.y * 20.,
                    MouseScrollUnit::Pixel => scroll.y,
                };

                inner_container.position += dy;
                inner_container.position = inner_container.position.clamp(-max_scroll, 0.);
                style.top = Val::Px(inner_container.position);
            },
        );
    }
}
