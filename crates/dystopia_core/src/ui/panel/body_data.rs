use bevy::{
    app::{App, Plugin, Update},
    core::Name,
    input::ButtonState,
    log::warn,
    prelude::{
        in_state, resource_exists, BuildChildren, ChildBuilder, Commands, Component, Deref, Entity,
        EventReader, EventWriter, Has, IntoSystemConfigs, MouseButton, NodeBundle, Query, Res,
        ResMut, Resource, TextBundle, Visibility,
    },
    text::{Text, TextStyle},
    ui::{AlignItems, FlexDirection, JustifyContent, PositionType, Style, Val},
};
use dystopia_derive::{AsBuiltComponent, LocalizableData};

use crate::{
    cosmos::celestial::{BodyIndex, BodyType, Cosmos, Moon, Planet, Star, StarType},
    distributed_list_element,
    input::{MouseInput, SceneMouseInput},
    localizable_enum,
    localization::{ui::LUiPanel, LangFile, Localizable, LocalizableData},
    merge_list,
    schedule::state::{GameState, SceneState},
    sci::unit::{Density, Illuminance, Length, Temperature, Time, Unit},
    ui::{
        ext::DefaultWithStyle,
        interation::{
            body_focus_button::BodyFocusButton,
            close_button::{ButtonClose, ButtonCloseStyle},
            scrollable_list::ScrollableList,
        },
        panel::PanelTargetChange,
        preset::{
            default_panel_style, default_section_style, default_title_style, FULLSCREEN_UI_CORNERS,
            PANEL_BACKGROUND, PANEL_BORDER_COLOR, PANEL_ELEM_TEXT_STYLE, PANEL_SUBTITLE_TEXT_STYLE,
            PANEL_TITLE_BACKGROUND, PANEL_TITLE_FONT_SIZE, PANEL_TITLE_HEIGHT,
            PANEL_TITLE_TEXT_COLOR,
        },
        update::AsBuiltComponent,
        GlobalUiRoot, UiAggregate, UiBuilder, UiStack, FUSION_PIXEL,
    },
};

localizable_enum!(LBodyType, pub, Star, Planet, Moon);
localizable_enum!(LDetailedBodyType, O, B, A, F, G, K, M, Rocky, Gas, Ice);
localizable_enum!(LBodyInfoType, Temperature, Density, Illuminance);
localizable_enum!(
    LBodyOrbitInfoType,
    ParentBody,
    OrbitRadius,
    SiderealPeriod,
    RotationPeriod
);
localizable_enum!(LBodyDataPanelSectionType, BodyInfo, OrbitInfo);

pub struct BodyDataPanelPlugin;

impl Plugin for BodyDataPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PanelTargetChange<BodyDataPanel>>()
            .add_systems(
                Update,
                (
                    potential_body_click_handler,
                    pack_body_data_panel_data,
                    update_ui_panel.run_if(resource_exists::<BodyDataPanel>),
                )
                    .run_if(in_state(SceneState::CosmosView)),
            )
            .add_systems(
                Update,
                on_target_change
                    .run_if(resource_exists::<BodyDataPanel>)
                    .run_if(in_state(GameState::Simulate)),
            );
    }
}

#[derive(Resource, Deref)]
pub struct BodyDataPanel(Entity);

#[derive(Component, AsBuiltComponent, LocalizableData)]
struct BodyDataPanelData {
    title: Localizable<LUiPanel>,

    section_body_info: Localizable<LBodyDataPanelSectionType>,
    #[lang_skip]
    body_name: String,
    body_ty: Localizable<LBodyType>,
    detailed_body_ty: Localizable<LDetailedBodyType>,
    title_temperature: Localizable<LBodyInfoType>,
    temperature: Localizable<Temperature>,
    title_density: Localizable<LBodyInfoType>,
    density: Localizable<Density>,
    title_illuminance: Localizable<LBodyInfoType>,
    illuminance: Localizable<Illuminance>,

    section_orbit_info: Localizable<LBodyDataPanelSectionType>,
    #[lang_skip]
    parent_body_index: Option<BodyIndex>,
    title_parent_body: Localizable<LBodyOrbitInfoType>,
    #[lang_skip]
    parent_body: String,
    title_orbit_radius: Localizable<LBodyOrbitInfoType>,
    orbit_radius: Localizable<Length>,
    title_sidereal_period: Localizable<LBodyOrbitInfoType>,
    sidereal_period: Localizable<Time>,
    title_rotation_period: Localizable<LBodyOrbitInfoType>,
    rotation_period: Localizable<Time>,
}

pub struct BodyDataPanelStyle {
    pub width: f32,
    pub height: f32,
}

impl UiAggregate for BodyDataPanelData {
    type Style = BodyDataPanelStyle;

    fn build(parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut entities = Vec::with_capacity(Self::NUM_FIELDS);

        let mut root = parent.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(style.width),
                    height: Val::Px(style.height),
                    bottom: FULLSCREEN_UI_CORNERS.bottom,
                    right: FULLSCREEN_UI_CORNERS.right,
                    ..default_panel_style()
                },
                background_color: PANEL_BACKGROUND,
                border_color: PANEL_BORDER_COLOR.into(),
                ..Default::default()
            },
            Name::new("BodyDataPanel"),
        ));
        let panel_entity = root.id();

        root.with_children(|root| {
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
                            text: Text::default_with_style(TextStyle {
                                font: FUSION_PIXEL,
                                font_size: PANEL_TITLE_FONT_SIZE,
                                color: PANEL_TITLE_TEXT_COLOR,
                            }),
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

            // Data
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ScrollableList,
            ))
            .with_children(|list_root| {
                // TODO Add a icon for different types of bodies.
                // Body info section
                list_root
                    .spawn(NodeBundle {
                        style: default_section_style(),
                        ..Default::default()
                    })
                    .with_children(|section_root| {
                        entities.push(
                            section_root
                                .spawn(TextBundle::default_with_style(PANEL_SUBTITLE_TEXT_STYLE))
                                .id(),
                        );

                        entities.extend(distributed_list_element!(
                            section_root,
                            // body_name
                            TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                            // body_ty
                            TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                            // detailed_body_ty
                            TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                        ));

                        entities.extend(merge_list!(
                            // temperature
                            distributed_list_element!(
                                section_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // density
                            distributed_list_element!(
                                section_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // illuminance
                            distributed_list_element!(
                                section_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            )
                        ));
                    });

                // Orbit info section
                list_root
                    .spawn(NodeBundle {
                        style: default_section_style(),
                        ..Default::default()
                    })
                    .with_children(|section_root| {
                        entities.push(
                            section_root
                                .spawn(TextBundle::default_with_style(PANEL_SUBTITLE_TEXT_STYLE))
                                .id(),
                        );

                        let mut focus =
                            section_root.spawn((NodeBundle::default(), BodyFocusButton::default()));
                        entities.push(focus.id());
                        focus.with_children(|focus_root| {
                            entities.extend(distributed_list_element!(
                                focus_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ));
                        });

                        entities.extend(merge_list!(
                            // orbit_radius
                            distributed_list_element!(
                                section_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // sidereal_period
                            distributed_list_element!(
                                section_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // rotation_period
                            distributed_list_element!(
                                section_root,
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            )
                        ));
                    });
            });
        })
        .insert(BuiltBodyDataPanelData::from_entities(entities))
        .id()
    }
}

fn potential_body_click_handler(
    clicked_query: Query<(Entity, &MouseInput, Option<&BodyIndex>)>,
    mut scene_mouse_input: EventReader<SceneMouseInput>,
    mut target_change: EventWriter<PanelTargetChange<BodyDataPanel>>,
) {
    for input in scene_mouse_input.read() {
        if !(input.button == MouseButton::Left && input.state == ButtonState::Pressed) {
            continue;
        }

        let Some((target, input, body)) = clicked_query.iter().nth(0) else {
            // Clicked on void
            target_change.send(PanelTargetChange::none());
            return;
        };

        if input.button == MouseButton::Left && input.state == ButtonState::Pressed {
            target_change.send(if body.is_none() {
                // Clicked on something that is not a body
                PanelTargetChange::none()
            } else {
                // Clicked on body!
                PanelTargetChange::some(target)
            });
        }
    }
}

fn pack_body_data_panel_data(
    mut commands: Commands,
    panel: Option<ResMut<BodyDataPanel>>,
    body_query: Query<(
        &Name,
        &BodyIndex,
        Has<Star>,
        Option<&StarType>,
        Has<Planet>,
        Has<Moon>,
        Option<&BodyType>,
    )>,
    cosmos: Res<Cosmos>,
    mut target_change: EventReader<PanelTargetChange<BodyDataPanel>>,
    global_root: Res<GlobalUiRoot>,
) {
    for target in target_change.read() {
        let Some(target) = **target else {
            if let Some(panel) = panel.as_deref() {
                commands.entity(**panel).insert(Visibility::Hidden);
            }
            continue;
        };

        let Ok((body_name, body_index, is_star, maybe_star_ty, is_planet, is_moon, maybe_body_ty)) =
            body_query.get(target)
        else {
            warn!("Failed to find the target body.");
            continue;
        };

        // body indices should be equal to orbit indices
        let Some(orbit) = cosmos.orbits.get(**body_index) else {
            warn!(
                "Failed to get orbit data for body {}, named {}",
                **body_index,
                body_name.as_str()
            );
            continue;
        };

        let body_ty = if is_star {
            LBodyType::Star
        } else if is_planet {
            LBodyType::Planet
        } else if is_moon {
            LBodyType::Moon
        } else {
            unreachable!()
        }
        .into();

        let detailed_body_ty = if let Some(st) = maybe_star_ty {
            match st {
                StarType::O => LDetailedBodyType::O,
                StarType::B => LDetailedBodyType::B,
                StarType::A => LDetailedBodyType::A,
                StarType::F => LDetailedBodyType::F,
                StarType::G => LDetailedBodyType::G,
                StarType::K => LDetailedBodyType::K,
                StarType::M => LDetailedBodyType::M,
            }
        } else if let Some(bt) = maybe_body_ty {
            match bt {
                BodyType::Rocky => LDetailedBodyType::Rocky,
                BodyType::GasGiant => LDetailedBodyType::Gas,
                BodyType::IceGiant => LDetailedBodyType::Ice,
            }
        } else {
            unreachable!()
        }
        .into();

        let parent_body = cosmos
            .entities
            .get(orbit.center_id)
            .and_then(|e| body_query.get(*e).map(|b| (b.0.to_string(), *b.1)).ok());

        let parameterized = &cosmos.parameterized[**body_index];

        let data = BodyDataPanelData {
            title: LUiPanel::BodyData.into(),

            section_body_info: LBodyDataPanelSectionType::BodyInfo.into(),
            body_name: body_name.to_string(),
            body_ty,
            detailed_body_ty,
            title_temperature: LBodyInfoType::Temperature.into(),
            temperature: Temperature::wrap_with_si(parameterized.temperature).into(),
            title_density: LBodyInfoType::Density.into(),
            density: Density::wrap_with_si(parameterized.density).into(),
            title_illuminance: LBodyInfoType::Illuminance.into(),
            illuminance: Illuminance::wrap_with_si(parameterized.illuminance).into(),

            section_orbit_info: LBodyDataPanelSectionType::OrbitInfo.into(),
            title_parent_body: LBodyOrbitInfoType::ParentBody.into(),
            parent_body_index: parent_body.clone().map(|(_, i)| i),
            parent_body: parent_body.map(|(n, _)| n).unwrap_or("None".to_owned()),
            title_orbit_radius: LBodyOrbitInfoType::OrbitRadius.into(),
            orbit_radius: Length::wrap_with_si(orbit.radius).into(),
            title_sidereal_period: LBodyOrbitInfoType::SiderealPeriod.into(),
            sidereal_period: Time::wrap_with_si(orbit.sidereal_period).into(),
            title_rotation_period: LBodyOrbitInfoType::RotationPeriod.into(),
            rotation_period: Time::wrap_with_si(orbit.rotation_period).into(),
        };

        if let Some(panel) = panel.as_deref() {
            commands.entity(**panel).insert(data);
        } else {
            let mut built = None;
            commands.entity(**global_root).with_children(|root| {
                built = Some(root.build_ui::<BodyDataPanelData>(BodyDataPanelStyle {
                    width: 250.,
                    height: 250.,
                }));
            });

            let panel = built.unwrap();
            commands.insert_resource(BodyDataPanel(panel));
            commands.entity(panel).insert(data);
        }
    }
}

fn update_ui_panel(
    mut commands: Commands,
    lang: Res<LangFile>,
    panel: ResMut<BodyDataPanel>,
    mut panel_query: Query<(&mut BodyDataPanelData, &BuiltBodyDataPanelData)>,
    mut stack: ResMut<UiStack>,
) {
    let panel = **panel;
    let Some((mut data, built)) = panel_query.get_mut(panel).ok() else {
        return;
    };

    data.localize(&lang);
    built.update(&data, &mut commands);
    stack.push(panel);
    commands.entity(panel).remove::<BodyDataPanelData>();
}

fn on_target_change(
    mut commands: Commands,
    mut target_change: EventReader<PanelTargetChange<BodyDataPanel>>,
    panel: ResMut<BodyDataPanel>,
) {
    for change in target_change.read() {
        commands.entity(**panel).insert(if change.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
