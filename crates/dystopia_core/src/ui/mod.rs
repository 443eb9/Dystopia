use std::ops::{Add, AddAssign, Sub, SubAssign};

use bevy::{
    app::{App, Plugin, Update},
    asset::{load_internal_binary_asset, Handle},
    math::Vec2,
    prelude::{
        in_state, ChildBuilder, Deref, DerefMut, Entity, FromWorld, IntoSystemConfigs, NodeBundle,
        Resource, World,
    },
    text::Font,
    ui::{Style, UiMaterialPlugin, Val},
};
use thiserror::Error;

use crate::{
    input::RayTransparent,
    math::{Axis, Direction},
    schedule::state::GameState,
    ui::{
        panel::{
            body_data::BodyDataPanelPlugin, scene_title::SceneTitlePlugin,
            system_statistics::SystemStatisticsPanelPlugin,
        },
        selecting::{
            body::{BodySelectingIconMaterial, BodySelectingIndicator},
            SelectingUiPlugin,
        },
        transition::MainTransitionablePlugin,
        update::MainUpdatablePlugin,
    },
};

pub mod button;
pub mod ext;
pub mod macros;
pub mod panel;
pub mod preset;
pub mod scrollable_list;
pub mod selecting;
pub mod sync;
pub mod transition;
pub mod update;

pub const FUSION_PIXEL: Handle<Font> = Handle::weak_from_u128(789641049865321367040365478967874510);

pub struct DystopiaUiPlugin;

impl Plugin for DystopiaUiPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            FUSION_PIXEL,
            "fusion-pixel-10px-monospaced.otf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );

        app.add_plugins((
            BodyDataPanelPlugin,
            SceneTitlePlugin,
            SystemStatisticsPanelPlugin,
        ))
        .add_plugins(SelectingUiPlugin)
        .add_plugins(UiMaterialPlugin::<BodySelectingIconMaterial>::default())
        .add_plugins((MainUpdatablePlugin, MainTransitionablePlugin))
        .add_systems(
            Update,
            (
                scrollable_list::init_structure,
                scrollable_list::handle_scroll,
                button::handle_button_close_click,
            )
                .run_if(in_state(GameState::Simulate)),
        )
        .add_systems(Update, (sync::scene_ui_sync, sync::cursor_ui_sync))
        .init_resource::<GlobalUiRoot>()
        .init_resource::<UiStack>();
    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<BodySelectingIndicator>();
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct UiStack(Vec<Entity>);

#[derive(Resource, Deref)]
pub struct GlobalUiRoot(Entity);

impl FromWorld for GlobalUiRoot {
    fn from_world(world: &mut World) -> Self {
        Self(
            world
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    RayTransparent,
                ))
                .id(),
        )
    }
}

pub trait UiAggregate {
    type Style;

    /// Build the ui and spawn them into world.
    fn build(parent: &mut ChildBuilder, style: Self::Style) -> Entity;
}

pub trait UiBuilder {
    fn build_ui<U: UiAggregate>(&mut self, style: U::Style) -> Entity;
}

impl UiBuilder for ChildBuilder<'_> {
    #[inline]
    fn build_ui<U: UiAggregate>(&mut self, style: U::Style) -> Entity {
        U::build(self, style)
    }
}

#[derive(Error, Debug)]
pub enum UiPosCreationError {
    #[error("Style value conflict on {0:?}.")]
    ValueConflict(Axis),
    #[error("Style value on {0:?} is not supported.")]
    ValueNotSupported(Direction),
}

#[derive(Clone, Copy)]
pub struct UiPos {
    /// Original position that depends on `original_param`
    pub pos: Vec2,
    pub dirs: [Direction; 2],
}

impl UiPos {
    pub fn new(style: &Style) -> Result<Self, UiPosCreationError> {
        let mut desc = [Direction::Up; 2];

        let x = {
            match style.left {
                Val::Auto => match style.right {
                    Val::Auto => {
                        desc[0] = Direction::Left;
                        0.
                    }
                    Val::Px(px) => {
                        desc[0] = Direction::Right;
                        px
                    }
                    _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Right)),
                },
                Val::Px(px) => {
                    if !matches!(style.right, Val::Auto) {
                        return Err(UiPosCreationError::ValueConflict(Axis::X));
                    }

                    desc[0] = Direction::Left;
                    px
                }
                _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Left)),
            }
        };

        let y = {
            match style.top {
                Val::Auto => match style.bottom {
                    Val::Auto => {
                        desc[1] = Direction::Up;
                        0.
                    }
                    Val::Px(px) => {
                        desc[1] = Direction::Down;
                        px
                    }
                    _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Down)),
                },
                Val::Px(px) => {
                    if !matches!(style.bottom, Val::Auto) {
                        return Err(UiPosCreationError::ValueConflict(Axis::Y));
                    }

                    desc[1] = Direction::Up;
                    px
                }
                _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Up)),
            }
        };

        Ok(Self {
            pos: Vec2 { x, y },
            dirs: desc,
        })
    }

    pub fn apply_to(self, style: &mut Style) {
        match self.dirs[0] {
            Direction::Left => style.left = Val::Px(self.pos.x),
            Direction::Right => style.right = Val::Px(self.pos.x),
            _ => unreachable!(),
        }

        match self.dirs[1] {
            Direction::Up => style.top = Val::Px(self.pos.y),
            Direction::Down => style.bottom = Val::Px(self.pos.y),
            _ => unreachable!(),
        }
    }
}

impl Add<Vec2> for UiPos {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: Vec2) -> Self::Output {
        match self.dirs[0] {
            Direction::Left => self.pos.x += rhs.x,
            Direction::Right => self.pos.x -= rhs.x,
            _ => unreachable!(),
        }

        match self.dirs[1] {
            Direction::Up => self.pos.y += rhs.y,
            Direction::Down => self.pos.y -= rhs.y,
            _ => unreachable!(),
        }

        self
    }
}

impl Sub<Vec2> for UiPos {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vec2) -> Self::Output {
        self + (-rhs)
    }
}

impl AddAssign<Vec2> for UiPos {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

impl SubAssign<Vec2> for UiPos {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
    }
}
