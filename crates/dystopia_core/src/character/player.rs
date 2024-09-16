use bevy::{
    app::{App, Plugin, Update},
    asset::AssetServer,
    ecs::query::QuerySingleError,
    math::Vec2,
    prelude::{
        in_state, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Query, Res,
        Transform, ViewVisibility, Visibility, With,
    },
    sprite::{Anchor, Sprite, SpriteBundle, TextureAtlas},
    time::{Real, Time},
};

use crate::{
    assets::texture::TextureAtlasLayouts,
    character::{MoveSpeed, MoveSpeedFactor, ISOMETRIC_VEL_FACTOR},
    input::event::{
        KeyboardEventCenter, PLAYER_MOVE_DOWN, PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT, PLAYER_MOVE_UP,
        TOGGLE_CAMERA_CONTROL_OVERRIDE,
    },
    schedule::state::GameState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerAction>().add_systems(
            Update,
            (handle_player_move, handle_player_action).run_if(in_state(GameState::Simulate)),
        );
    }
}

#[derive(Event)]
pub enum PlayerAction {
    Change(Entity),
    Teleport(Vec2),
    ChangeVisibility(Visibility),
}

pub fn handle_player_action(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    layouts: Res<TextureAtlasLayouts>,
    mut player_query: Query<(Entity, &mut Transform, &mut Visibility), With<Player>>,
    mut actions: EventReader<PlayerAction>,
) {
    for action in actions.read() {
        match *action {
            PlayerAction::Change(entity) => {
                if let Ok((entity, ..)) = player_query.get_single() {
                    commands.entity(entity).remove::<Player>();
                }
                commands.entity(entity).insert(Player);
            }
            PlayerAction::Teleport(pos) => match player_query.get_single_mut() {
                Ok((_, mut transform, _)) => transform.translation = pos.extend(1.),
                Err(err) => match err {
                    QuerySingleError::NoEntities(_) => {
                        commands.spawn((
                            SpriteBundle {
                                sprite: Sprite {
                                    anchor: Anchor::BottomCenter,
                                    ..Default::default()
                                },
                                texture: asset_server.load("images/characters.png"),
                                transform: Transform::from_translation(pos.extend(1.)),
                                ..Default::default()
                            },
                            TextureAtlas {
                                layout: layouts.characters.clone(),
                                index: 0,
                            },
                            Player,
                            MoveSpeed::new(20.),
                            MoveSpeedFactor::default(),
                        ));
                    }
                    QuerySingleError::MultipleEntities(_) => {
                        panic!("Multiple `Player` detected.")
                    }
                },
            },
            PlayerAction::ChangeVisibility(target) => {
                if let Ok((_, _, mut vis)) = player_query.get_single_mut() {
                    *vis = target;
                }
            }
        }
    }
}

/// Mark an entity as player. There should be at most 1 player at one time.
#[derive(Component)]
pub struct Player;

fn handle_player_move(
    mut player_query: Query<
        (
            &mut Transform,
            &MoveSpeed,
            &MoveSpeedFactor,
            &ViewVisibility,
        ),
        With<Player>,
    >,
    event_center: Res<KeyboardEventCenter>,
    time: Res<Time<Real>>,
) {
    let Ok((mut transform, speed, factor, vis)) = player_query.get_single_mut() else {
        return;
    };

    if !vis.get() || event_center.is_activating(TOGGLE_CAMERA_CONTROL_OVERRIDE) {
        return;
    }

    let mut vel = Vec2::ZERO;
    if event_center.is_activating(PLAYER_MOVE_UP) {
        vel.y += 1.;
    }
    if event_center.is_activating(PLAYER_MOVE_DOWN) {
        vel.y -= 1.;
    }
    if event_center.is_activating(PLAYER_MOVE_LEFT) {
        vel.x -= 1.;
    }
    if event_center.is_activating(PLAYER_MOVE_RIGHT) {
        vel.x += 1.;
    }

    transform.translation +=
        (**speed * **factor * time.delta_seconds() * vel * ISOMETRIC_VEL_FACTOR).extend(0.);
}
