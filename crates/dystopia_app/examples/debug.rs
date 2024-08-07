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
    cosmos::{celestial::BodyIndex, gen::CosmosGenerationSettings},
    distributed_list_element,
    input::camera::CameraBehavior,
    map::{
        bundle::{TileBundle, TilemapBundle},
        tilemap::{
            FlattenedTileIndex, TileAtlasIndex, TileBindedTilemap, TileFlip, TileIndex,
            TileRenderSize, TilemapStorage, TilemapTexture, TilemapTextureDescriptor,
            TilemapTilesets,
        },
    },
    math::shape::icosahedron,
    schedule::state::{AssetState, GameState},
    sci::unit::Length,
    simulation::{MainCamera, ViewScale},
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
                physics_debug: true,
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
            // .add_systems(OnEnter(GameState::Simulate), debug_tilemap)
            // .add_systems(Update, debug_rm_vis)
            .add_systems(Startup, setup_debug)
            .add_systems(Update, toggle_ui_debug);
    }
}

fn setup_debug(mut commands: Commands) {}

fn debug_skip_menu(mut commands: Commands, mut game_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(CosmosGenerationSettings {
        seed: 2,
        galaxy_radius: Length::LightYear(1.),
        // num_stars: 60..69,
        num_stars: 1..2,
    });
    game_state.set(GameState::Initialize);
    info!("Skipped menu");
}

fn debug_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    const CHUNK_SIZE: u32 = 8;

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        tile_render_size: TileRenderSize(Vec2::splat(32.)),
        storgae: TilemapStorage::new(entity, CHUNK_SIZE),
        tilesets: TilemapTilesets::new(
            vec![
                TilemapTexture {
                    handle: asset_server.load("images/test_tileset_a.png"),
                    desc: TilemapTextureDescriptor {
                        size: UVec2 { x: 45, y: 26 },
                        tile_size: UVec2 { x: 15, y: 13 },
                    },
                },
                TilemapTexture {
                    handle: asset_server.load("images/test_tileset_b.png"),
                    desc: TilemapTextureDescriptor {
                        size: UVec2 { x: 45, y: 26 },
                        tile_size: UVec2 { x: 15, y: 13 },
                    },
                },
            ],
            FilterMode::Nearest,
        ),
        ..Default::default()
    };

    // let anim_dn = tilemap.animations.register(
    //     vec![
    //         (0, 0),
    //         (0, 1),
    //         (0, 2),
    //         (1, 1),
    //         (1, 0),
    //         (0, 1),
    //         (1, 2),
    //         (0, 1),
    //     ],
    //     3,
    // );
    // let anim_up = tilemap.animations.register(
    //     vec![
    //         (0, 3),
    //         (0, 4),
    //         (0, 5),
    //         (1, 4),
    //         (1, 3),
    //         (0, 4),
    //         (1, 5),
    //         (0, 4),
    //     ],
    //     3,
    // );

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
    for (i_tri, tri) in icosahedron(2, IVec3::Y).into_iter().enumerate() {
        let texture = if i_tri % 2 == 0 { 0 } else { 1 };
        let atlas = if tri.element_sum() == 1 { 0 } else { 3 };
        tilemap.storgae.set(
            &mut commands,
            TileBundle {
                binded_tilemap: TileBindedTilemap(entity),
                index: TileIndex::new(tri, CHUNK_SIZE),
                // atlas_index: TileAtlasIndex::Static((texture, atlas, TileFlip::HORIZONTAL).into()),
                atlas_index: TileAtlasIndex::Animated {
                    anim: if tri.element_sum() == 1 {
                        anim_dn
                    } else {
                        anim_up
                    },
                    offset_milisec: rng.gen_range(0..2000),
                    // offset_milisec: 0,
                },
                ..Default::default()
            },
        );
    }

    commands.entity(entity).insert(tilemap);
}

fn debug_rm_vis(
    mut commands: Commands,
    mut tilemaps_query: Query<&mut TilemapStorage>,
    mut tiles_query: Query<&mut Visibility, With<TileIndex>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: Local<Visibility>,
    time: Res<Time<Real>>,
    mut twinkled: Local<bool>,
) {
    let mut rng = rand::thread_rng();

    if keyboard.just_pressed(KeyCode::Digit1) {
        for mut storage in &mut tilemaps_query {
            for tri in icosahedron(2, IVec3::Y) {
                if rng.gen_range(0.0..1.0) > 0.5 {
                    storage.remove(&mut commands, tri);
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Digit2) {
        for mut storage in &mut tilemaps_query {
            let chunks = icosahedron(2, IVec3::Y)
                .into_iter()
                .map(|i| FlattenedTileIndex::from_direct(i, storage.chunk_size()).chunk_index)
                .collect::<HashSet<_>>();

            for chunk in chunks {
                if rng.gen_range(0.0..1.0) > 0.5 {
                    storage.remove_chunk(&mut commands, chunk);
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Digit3) {
        for storage in &tilemaps_query {
            storage.despawn(&mut commands);
        }
    }

    if keyboard.pressed(KeyCode::Digit4) {
        const CYCLE: f32 = 0.25;
        let t = (time.elapsed_seconds() / CYCLE) as u32 % 2 == 0;

        if t != *twinkled {
            return;
        }

        if *visible != Visibility::Hidden {
            tiles_query.iter_mut().for_each(|mut visibility| {
                *visibility = Visibility::Inherited;
            });
            *visible = Visibility::Hidden;
        } else {
            tiles_query.iter_mut().for_each(|mut visibility| {
                if rng.gen_range(0.0..1.0) > 0.5 {
                    *visibility = Visibility::Hidden;
                }
            });
            *visible = Visibility::Inherited;
        }

        *twinkled = !t;
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
