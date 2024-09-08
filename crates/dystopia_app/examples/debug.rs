#![allow(unused)]

use avian2d::{prelude::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::{
    app::{App, Plugin, PluginGroup, Startup, Update},
    asset::AssetServer,
    color::palettes::css::WHITE,
    dev_tools::ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::ButtonInput,
    log::info,
    math::{IVec3, UVec2, Vec2},
    prelude::{
        BuildChildren, Camera2dBundle, Changed, Commands, Entity, KeyCode, Local, NodeBundle,
        Query, Res, ResMut, TextBundle, With,
    },
    render::{
        camera::OrthographicProjection,
        render_resource::FilterMode,
        settings::{Backends, RenderCreation, WgpuSettings},
        view::Visibility,
        RenderPlugin,
    },
    state::state::{NextState, OnEnter},
    text::{Text, TextStyle},
    time::{Real, Time},
    ui::{FlexDirection, FlexWrap, IsDefaultUiCamera, Overflow, Style, Val},
    utils::hashbrown::HashSet,
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use dystopia_core::{
    cosmos::{
        celestial::{BodyIndex, BodyTilemap, ToLoadTilemap, ToSaveTilemap},
        gen::CosmosGenerationSettings,
    },
    distributed_list_element,
    input::{camera::CameraBehavior, MouseClickCounter, MouseInput},
    map::{
        bundle::TilemapBundle,
        shape::rectangle,
        tilemap::{
            FlattenedTileIndex, Tile, TileAtlasIndex, TileFlip, TileIndex, TileRenderSize,
            TilemapStorage, TilemapTexture, TilemapTextureDescriptor, TilemapTilesets,
        },
    },
    schedule::state::{AssetState, GameState, SceneState},
    sci::unit::Length,
    sim::{MainCamera, SaveName, ViewScale},
    ui::{
        panel::body_data::BodyDataPanel, scrollable_list::ScrollableList, UiBuilder, FUSION_PIXEL,
    },
    DystopiaCorePlugin,
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoVsync,
                        title: "Dystopia".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            DystopiaCorePlugin,
            DystopiaDebugPlugin {
                inspector: true,
                ui_debug: true,
                physics_debug: false,
            },
            PhysicsPlugins::default(),
        ))
        .run();
}

pub struct DystopiaDebugPlugin {
    pub inspector: bool,
    pub ui_debug: bool,
    pub physics_debug: bool,
}

impl Plugin for DystopiaDebugPlugin {
    fn build(&self, app: &mut App) {
        if self.inspector {
            app.add_plugins(WorldInspectorPlugin::default());
        }

        if self.ui_debug {
            app.add_plugins(DebugUiPlugin);
        }

        if self.physics_debug {
            app.add_plugins(PhysicsDebugPlugin::default());
        }

        app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
            .add_systems(OnEnter(AssetState::Finish), debug_skip_menu)
            // .add_systems(OnEnter(GameState::Simulate), debug_ui)
            .add_systems(OnEnter(GameState::Simulate), debug_tilemap)
            // .add_systems(Update, debug_rm_vis)
            .add_systems(Startup, setup_debug)
            .add_systems(Update, toggle_ui_debug)
            // .add_systems(Update, test_multi_click)
            .add_systems(Update, debug_rm_vis);
    }
}

fn setup_debug(mut commands: Commands) {}

fn test_multi_click(query: Query<&MouseClickCounter, With<MouseInput>>) {
    for click in &query {
        dbg!(**click);
    }
}

fn debug_skip_menu(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut scene_state: ResMut<NextState<SceneState>>,
) {
    commands.insert_resource(CosmosGenerationSettings {
        seed: 5,
        galaxy_radius: Length::LightYear(1.),
        // num_stars: 60..69,
        num_stars: 1..2,
    });
    game_state.set(GameState::Initialize);
    scene_state.set(SceneState::CosmosView);
    commands.insert_resource(SaveName::new("debug_save".to_string()));
    info!("Skipped menu");
}

fn debug_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bodies: Query<Entity, With<BodyIndex>>,
) {
    const CHUNK_SIZE: u32 = 8;

    let mut tilemap = TilemapBundle {
        tile_render_size: TileRenderSize(Vec2 { x: 32., y: 16. }),
        storgae: TilemapStorage::new(CHUNK_SIZE),
        tilesets: TilemapTilesets::new(
            vec![
                TilemapTexture {
                    handle: asset_server.load("images/test_tileset_a.png"),
                    desc: TilemapTextureDescriptor {
                        size: UVec2 { x: 32, y: 32 },
                        tile_size: UVec2 { x: 32, y: 16 },
                    },
                },
                TilemapTexture {
                    handle: asset_server.load("images/test_tileset_b.png"),
                    desc: TilemapTextureDescriptor {
                        size: UVec2 { x: 32, y: 32 },
                        tile_size: UVec2 { x: 32, y: 16 },
                    },
                },
            ],
            FilterMode::Nearest,
        ),
        ..Default::default()
    };

    let anim_dn = tilemap.animations.register(
        vec![
            (0, 0, TileFlip::NONE),
            (0, 1, TileFlip::NONE),
            (0, 2, TileFlip::HORIZONTAL),
            (1, 1, TileFlip::NONE),
        ],
        3,
    );
    let anim_up = tilemap.animations.register(
        vec![
            (0, 3, TileFlip::NONE),
            (0, 4, TileFlip::NONE),
            (0, 5, TileFlip::HORIZONTAL),
            (1, 4, TileFlip::NONE),
        ],
        3,
    );

    let mut rng = rand::thread_rng();
    for (i_tile, index) in rectangle(10, 10).into_iter().enumerate() {
        tilemap.storgae.set(Tile {
            index: TileIndex::from_direct(index.as_ivec2(), CHUNK_SIZE),
            atlas_index: TileAtlasIndex::Static((0, 0).into()),
            ..Default::default()
        });
    }

    let tilemap = commands.spawn(tilemap).id();
    if let Some(body) = bodies.iter().nth(0) {
        commands.entity(body).insert(BodyTilemap::new(tilemap));
    } else {
        commands.spawn(BodyTilemap::new(tilemap));
    }
}

fn debug_rm_vis(
    mut commands: Commands,
    mut tilemaps_query: Query<(Entity, Option<&mut TilemapStorage>, &BodyIndex)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: Local<bool>,
    time: Res<Time<Real>>,
    mut twinkled: Local<bool>,
) {
    for (entity, mut storage, _) in &mut tilemaps_query {
        if keyboard.just_pressed(KeyCode::Digit5) {
            commands.entity(entity).insert(ToSaveTilemap {
                remove_after_done: true,
            });
        }

        if keyboard.just_pressed(KeyCode::Digit6) {
            commands.entity(entity).insert(ToLoadTilemap);
        }

        let Some(mut storage) = storage else {
            return;
        };

        let mut rng = rand::thread_rng();

        if keyboard.just_pressed(KeyCode::Digit1) {
            for tile in rectangle(16, 9) {
                if rng.gen_range(0.0..1.0) > 0.5 {
                    storage.remove(tile.as_ivec2());
                }
            }
        }

        if keyboard.just_pressed(KeyCode::Digit2) {
            let chunks = rectangle(16, 9)
                .into_iter()
                .map(|i| {
                    FlattenedTileIndex::from_direct(i.as_ivec2(), storage.chunk_size()).in_chunk
                })
                .collect::<HashSet<_>>();

            for chunk in chunks {
                if rng.gen_range(0.0..1.0) > 0.5 {
                    storage.remove_chunk(chunk);
                }
            }
        }

        if keyboard.just_pressed(KeyCode::Digit3) {
            storage.clear();
        }

        if keyboard.pressed(KeyCode::Digit4) {
            const CYCLE: f32 = 0.25;
            let t = (time.elapsed_seconds() / CYCLE) as u32 % 2 == 0;

            if t != *twinkled {
                return;
            }

            unsafe {
                let cell = storage.as_unsafe_cell();
                if *visible {
                    (*cell.internal).values_mut().for_each(|c| {
                        c.iter_mut().filter_map(|t| t.as_mut()).for_each(|t| {
                            t.visible = *visible;
                        });
                    });
                } else {
                    (*cell.internal).values_mut().for_each(|c| {
                        c.iter_mut().filter_map(|t| t.as_mut()).for_each(|t| {
                            if rng.gen_range(0.0..1.0) > 0.5 {
                                t.visible = false;
                            }
                        });
                    });
                }
                *visible = !*visible;
                (*cell.changed_chunks).extend((*cell.internal).keys());
            }

            *twinkled = !t;
        }
    }
}

fn toggle_ui_debug(keyboard: Res<ButtonInput<KeyCode>>, mut options: ResMut<UiDebugOptions>) {
    if keyboard.just_pressed(KeyCode::Space) {
        info!("Toggle ui debug");
        options.toggle();
    }
}

fn debug_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|root| {
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(250.),
                        height: Val::Px(500.),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ScrollableList,
            ))
            .with_children(|list_root| {
                for i in 0..30 {
                    distributed_list_element!(
                        list_root,
                        Default::default(),
                        TextBundle::from_section(
                            format!("Test Elem {}", i),
                            TextStyle {
                                font: FUSION_PIXEL,
                                font_size: 20.,
                                color: WHITE.into()
                            }
                        ),
                        TextBundle::from_section(
                            format!("{}", i),
                            TextStyle {
                                font: FUSION_PIXEL,
                                font_size: 20.,
                                color: WHITE.into()
                            }
                        )
                    );
                }
            });
        });
}
