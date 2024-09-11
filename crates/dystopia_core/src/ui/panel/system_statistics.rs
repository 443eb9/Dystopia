use bevy::{
    app::{App, Plugin, Update},
    core::Name,
    prelude::{
        in_state, resource_exists, BuildChildren, ChildBuilder, Commands, Component, Deref, Entity,
        EventReader, EventWriter, Has, IntoSystemConfigs, NodeBundle, Query, Res, ResMut, Resource,
        TextBundle, Transform, Visibility, With, Without,
    },
    text::Text,
    ui::{AlignItems, FlexDirection, JustifyContent, PositionType, Style, Val},
};
use dystopia_derive::{AsBuiltComponent, LocalizableData};

use crate::{
    cosmos::celestial::{BodyIndex, Cosmos, Moon, Planet, Star, System},
    distributed_list_element,
    input::MouseInput,
    localization::{ui::LUiPanel, LangFile, Localizable, LocalizableData},
    schedule::state::{GameState, SceneState},
    sim::MainCamera,
    ui::{
        ext::DefaultWithStyle,
        interation::close_button::{ButtonClose, ButtonCloseStyle},
        panel::{
            body_data::{BodyDataPanel, LBodyType},
            PanelTargetChange,
        },
        preset::{
            default_section_style, default_title_style, FULLSCREEN_UI_CORNERS, PANEL_BACKGROUND,
            PANEL_BORDER, PANEL_BORDER_COLOR, PANEL_ELEM_TEXT_STYLE, PANEL_SUBTITLE_TEXT_STYLE,
            PANEL_TITLE_BACKGROUND, PANEL_TITLE_HEIGHT, PANEL_TITLE_TEXT_STYLE,
        },
        update::{
            AsBuiltComponent, AsOriginalComponent, AsUpdatableData, DataUpdatableUi,
            UpdatablePlugin,
        },
        GlobalUiRoot, UiAggregate, UiBuilder,
    },
};

pub struct SystemStatisticsPanelPlugin;

impl Plugin for SystemStatisticsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UpdatablePlugin::<BuiltBodyInfo, BodyInfo>::default())
            .add_systems(
                Update,
                (
                    pack_system_statistics_data,
                    update_system_statistics_data.run_if(resource_exists::<SystemStatisticsPanel>),
                    handle_body_info_click,
                )
                    .run_if(in_state(SceneState::CosmosView)),
            )
            .add_systems(
                Update,
                on_target_change
                    .run_if(resource_exists::<SystemStatisticsPanel>)
                    .run_if(in_state(GameState::Simulate)),
            );
    }
}

#[derive(Clone, AsBuiltComponent, LocalizableData)]
struct BodyInfo {
    #[lang_skip]
    name: String,
    ty: Localizable<LBodyType>,
}

#[derive(Component)]
struct BodyInfoButton {
    target: Option<BodyIndex>,
}

impl AsUpdatableData for BodyInfo {
    type UpdatableData = Self;
}

impl AsOriginalComponent for BodyInfo {
    type OriginalComponent = BuiltBodyInfo;
}

impl DataUpdatableUi<BodyInfo> for BuiltBodyInfo {
    fn update_data(&mut self, data: &BodyInfo, commands: &mut Commands) {
        self.update(data, commands);
    }
}

impl UiAggregate for BodyInfo {
    type Style = ();

    fn build(parent: &mut ChildBuilder, _style: Self::Style) -> Entity {
        let mut entities = Vec::with_capacity(Self::NUM_FIELDS);

        parent
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                BodyInfoButton { target: None },
            ))
            .with_children(|root| {
                entities.extend(distributed_list_element!(
                    root,
                    TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                    TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                ));
            })
            .insert(BuiltBodyInfo::from_entities(entities))
            .id()
    }
}

#[derive(Resource, Deref)]
pub struct SystemStatisticsPanel(Entity);

#[derive(Component, AsBuiltComponent, LocalizableData)]
pub struct SystemStatisticsPanelData {
    title: Localizable<LUiPanel>,
    #[lang_skip]
    name: String,
    #[dynamic_sized(10)]
    bodies: Vec<BodyInfo>,
}

pub struct SystemStatisticsPanelStyle {
    pub width: f32,
}

impl UiAggregate for SystemStatisticsPanelData {
    type Style = SystemStatisticsPanelStyle;

    fn build(parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut entities = Vec::with_capacity(Self::NUM_FIELDS);

        let mut root = parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(style.width),
                height: Val::Percent(100.),
                left: FULLSCREEN_UI_CORNERS.left,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        });
        let panel_entity = root.id();

        root.with_children(|side_bar| {
            side_bar
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(50.),
                        flex_direction: FlexDirection::Column,
                        border: PANEL_BORDER,
                        ..Default::default()
                    },
                    background_color: PANEL_BACKGROUND,
                    border_color: PANEL_BORDER_COLOR.into(),
                    ..Default::default()
                })
                .with_children(|root| {
                    // Title Bar
                    root.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Px(PANEL_TITLE_HEIGHT),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default_title_style()
                        },
                        background_color: PANEL_TITLE_BACKGROUND,
                        ..Default::default()
                    })
                    .with_children(|title_root| {
                        entities.push(
                            title_root
                                .spawn(TextBundle {
                                    text: Text::default_with_style(PANEL_TITLE_TEXT_STYLE),
                                    ..Default::default()
                                })
                                .id(),
                        );

                        ButtonClose::build(
                            title_root,
                            ButtonCloseStyle {
                                size: Val::Px(PANEL_TITLE_HEIGHT - 5.),
                                target: panel_entity,
                            },
                        );
                    });

                    root.spawn((
                        NodeBundle {
                            style: default_section_style(),
                            ..Default::default()
                        },
                        // ScrollableList,
                    ))
                    .with_children(|data| {
                        // System Name
                        entities.push(
                            data.spawn(TextBundle::default_with_style(PANEL_SUBTITLE_TEXT_STYLE))
                                .id(),
                        );

                        // Bodies
                        entities.extend((0..10).flat_map(|_| {
                            let mut e = Vec::new();
                            data.spawn(NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|body_info| {
                                e.push(body_info.build_ui::<BodyInfo>(()));
                            });
                            e
                        }));
                    });
                });
        })
        .insert(BuiltSystemStatisticsPanelData::from_entities(entities))
        .id()
    }
}

fn pack_system_statistics_data(
    mut commands: Commands,
    cosmos: Res<Cosmos>,
    mut target_change: EventReader<PanelTargetChange<BodyDataPanel>>,
    bodies_query: Query<(Entity, &Name, &System, Has<Star>, Has<Planet>, Has<Moon>)>,
    panel: Option<Res<SystemStatisticsPanel>>,
    global_root: Res<GlobalUiRoot>,
) {
    for target in target_change.read() {
        let Some((_, name, system, is_star, ..)) = target.and_then(|t| bodies_query.get(t).ok())
        else {
            continue;
        };

        if !is_star {
            continue;
        }

        let data = SystemStatisticsPanelData {
            title: LUiPanel::SystemStatistics.into(),
            name: name.to_string(),
            bodies: system
                .iter()
                .enumerate()
                .map(|(index, body)| {
                    let (_, name, _, is_star, is_planet, is_moon) =
                        bodies_query.get(cosmos.entities[**body]).unwrap();
                    BodyInfo {
                        name: format!("#{} {}", index, name),
                        ty: if is_star {
                            LBodyType::Star.into()
                        } else if is_planet {
                            LBodyType::Planet.into()
                        } else if is_moon {
                            LBodyType::Moon.into()
                        } else {
                            unreachable!()
                        },
                    }
                })
                .collect(),
        };

        if let Some(built) = panel.as_deref() {
            commands.entity(**built).insert(data);
        } else {
            let mut built = None;
            commands.entity(**global_root).with_children(|root| {
                built = Some(root.build_ui::<SystemStatisticsPanelData>(
                    SystemStatisticsPanelStyle { width: 250. },
                ));
            });

            let panel = built.unwrap();
            commands.insert_resource(SystemStatisticsPanel(panel));
            commands.entity(panel).insert(data);
        }
    }
}

fn update_system_statistics_data(
    mut commands: Commands,
    panel: Res<SystemStatisticsPanel>,
    lang: Res<LangFile>,
    mut panel_query: Query<(
        &mut SystemStatisticsPanelData,
        &BuiltSystemStatisticsPanelData,
    )>,
) {
    let Ok((mut data, built)) = panel_query.get_mut(**panel) else {
        return;
    };

    data.localize(&lang);
    built.update(&data, &mut commands);
    commands
        .entity(**panel)
        .remove::<SystemStatisticsPanelData>();
}

fn on_target_change(
    mut commands: Commands,
    mut target_change: EventReader<PanelTargetChange<BodyDataPanel>>,
    panel: ResMut<SystemStatisticsPanel>,
    bodies_query: Query<(Has<Star>, &System), With<BodyIndex>>,
    panel_query: Query<&BuiltSystemStatisticsPanelData>,
) {
    for change in target_change.read() {
        let vis = {
            if let Some(change) = **change {
                if let Ok((is_star, system)) = bodies_query.get(change) {
                    if is_star {
                        let panel = panel_query.get(**panel).unwrap();
                        panel
                            .bodies
                            .iter()
                            .take(system.len())
                            .zip(system.iter())
                            .for_each(|(btn, body)| {
                                commands.entity(*btn).insert(BodyInfoButton {
                                    target: Some(*body),
                                });
                            });
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                } else {
                    Visibility::Hidden
                }
            } else {
                Visibility::Hidden
            }
        };

        commands.entity(**panel).insert(vis);
    }
}

fn handle_body_info_click(
    mut main_camera_query: Query<&mut Transform, (With<MainCamera>, Without<BodyIndex>)>,
    bodies_query: Query<&Transform, With<BodyIndex>>,
    btn_query: Query<(&BodyInfoButton, &MouseInput)>,
    mut target_change: EventWriter<PanelTargetChange<BodyDataPanel>>,
    cosmos: Res<Cosmos>,
) {
    for (btn, input) in &btn_query {
        if !input.is_left_click() {
            continue;
        }

        if let Some(target) = btn.target {
            let entity = cosmos.entities[*target];
            target_change.send(PanelTargetChange::some(entity));
            main_camera_query.single_mut().translation =
                bodies_query.get(entity).unwrap().translation;
        }
    }
}
