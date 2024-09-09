use bevy::{
    app::{App, Plugin, Update},
    core::Name,
    input::ButtonState,
    log::warn,
    prelude::{
        in_state, resource_exists, BuildChildren, ChildBuilder, Commands, Component, Deref,
        DerefMut, Entity, EventReader, EventWriter, IntoSystemConfigs, MouseButton, NodeBundle,
        Query, Res, ResMut, Resource, TextBundle, Visibility,
    },
    text::{Text, TextStyle},
    ui::{AlignItems, FlexDirection, JustifyContent, Style, Val},
};
use dystopia_derive::{AsBuiltComponent, LocalizableEnum, LocalizableStruct};

use crate::{
    cosmos::celestial::{BodyIndex, BodyType, Cosmos, Moon, Planet, Star, StarType},
    distributed_list_element, gen_localizable_enum,
    input::{MouseInput, SceneMouseInput},
    localization::{ui::LUiPanel, LangFile, LocalizableDataWrapper, LocalizableStruct},
    merge_list,
    schedule::state::{GameState, SceneState},
    sci::unit::{Length, Time, Unit},
    ui::{
        button::{ButtonClose, ButtonCloseStyle},
        ext::DefaultWithStyle,
        panel::PanelTargetChange,
        preset::{
            default_panel_style, default_section_style, default_title_style, FULLSCREEN_UI_CORNERS,
            PANEL_BACKGROUND, PANEL_BORDER_COLOR, PANEL_ELEM_TEXT_STYLE, PANEL_SUBTITLE_TEXT_STYLE,
            PANEL_TITLE_BACKGROUND, PANEL_TITLE_FONT_SIZE, PANEL_TITLE_HEIGHT,
            PANEL_TITLE_TEXT_COLOR, SECTION_MARGIN,
        },
        primitive::AsBuiltComponent,
        scrollable_list::ScrollableList,
        GlobalUiRoot, UiAggregate, UiBuilder, UiStack, FUSION_PIXEL,
    },
};

#[derive(Resource, Deref, DerefMut)]
pub struct BodyDataPanel(Entity);

gen_localizable_enum!(LBodyType, Star, Planet, Moon);
gen_localizable_enum!(LDetailedBodyType, O, B, A, F, G, K, M, Rocky, Gas, Ice);
gen_localizable_enum!(
    LBodyOrbitInfoType,
    ParentBody,
    OrbitRadius,
    SiderealPeriod,
    RotationPeriod
);
gen_localizable_enum!(LBodyDataPanelSectionType, BodyInfo, OrbitInfo);

pub struct BodyDataPanelPlugin;

impl Plugin for BodyDataPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PanelTargetChange<BodyDataPanel>>()
            .add_systems(
                Update,
                (
                    potential_body_click_handler,
                    pack_body_data_panel_data,
                    update_ui_panel_data.run_if(resource_exists::<BodyDataPanel>),
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

#[derive(Component, AsBuiltComponent, LocalizableStruct)]
struct BodyDataPanelData {
    title: LocalizableDataWrapper<LUiPanel>,

    section_body_info: LocalizableDataWrapper<LBodyDataPanelSectionType>,
    #[lang_skip]
    body_name: String,
    body_ty: LocalizableDataWrapper<LBodyType>,
    detailed_body_ty: LocalizableDataWrapper<LDetailedBodyType>,

    section_orbit_info: LocalizableDataWrapper<LBodyDataPanelSectionType>,
    title_parent_body: LocalizableDataWrapper<LBodyOrbitInfoType>,
    #[lang_skip]
    parent_body: String,
    title_orbit_radius: LocalizableDataWrapper<LBodyOrbitInfoType>,
    orbit_radius: LocalizableDataWrapper<Length>,
    title_sidereal_period: LocalizableDataWrapper<LBodyOrbitInfoType>,
    sidereal_period: LocalizableDataWrapper<Time>,
    title_rotation_period: LocalizableDataWrapper<LBodyOrbitInfoType>,
    rotation_period: LocalizableDataWrapper<Time>,
}

pub struct BodyDataPanelStyle {
    pub width: f32,
    pub height: f32,
}

impl UiAggregate for BodyDataPanelData {
    type Style = BodyDataPanelStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut entities = Vec::with_capacity(Self::NUM_FIELDS);

        let mut root = parent.spawn((
            NodeBundle {
                style: Style {
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

                ButtonClose.build(
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
                        height: Val::Px(style.height - PANEL_TITLE_HEIGHT - SECTION_MARGIN),
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
                            Val::Percent(20.),
                            // body_name
                            TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                            // body_ty
                            TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                            // detailed_body_ty
                            TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
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

                        entities.extend(merge_list!(
                            // parent_body
                            distributed_list_element!(
                                section_root,
                                Val::Px(20.),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // orbit_radius
                            distributed_list_element!(
                                section_root,
                                Val::Px(20.),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // sidereal_period
                            distributed_list_element!(
                                section_root,
                                Val::Px(20.),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE),
                                TextBundle::default_with_style(PANEL_ELEM_TEXT_STYLE)
                            ),
                            // rotation_period
                            distributed_list_element!(
                                section_root,
                                Val::Px(20.),
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
        Option<&Star>,
        Option<&StarType>,
        Option<&Planet>,
        Option<&Moon>,
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

        let Ok((
            body_name,
            body_index,
            maybe_star,
            maybe_star_ty,
            maybe_planet,
            maybe_moon,
            maybe_body_ty,
        )) = body_query.get(target)
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

        let body_ty = if maybe_star.is_some() {
            LBodyType::Star
        } else if maybe_planet.is_some() {
            LBodyType::Planet
        } else if maybe_moon.is_some() {
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
            .map(|e| body_query.get(*e).unwrap().0.to_string())
            .unwrap_or_else(|| "None".to_string());

        let data = BodyDataPanelData {
            title: LUiPanel::BodyData.into(),
            section_body_info: LBodyDataPanelSectionType::BodyInfo.into(),
            body_name: body_name.to_string(),
            body_ty,
            detailed_body_ty,
            section_orbit_info: LBodyDataPanelSectionType::OrbitInfo.into(),
            title_parent_body: LBodyOrbitInfoType::ParentBody.into(),
            parent_body,
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
                built = Some(root.build_ui(
                    &data,
                    BodyDataPanelStyle {
                        width: 250.,
                        height: 250.,
                    },
                ));
            });

            let panel = built.unwrap();
            commands.insert_resource(BodyDataPanel(panel));
            commands.entity(panel).insert(data);
        }
    }
}

fn update_ui_panel_data(
    mut commands: Commands,
    lang: Res<LangFile>,
    panel: ResMut<BodyDataPanel>,
    mut panel_query: Query<(Entity, &mut BodyDataPanelData, &BuiltBodyDataPanelData)>,
    mut stack: ResMut<UiStack>,
) {
    let Some((entity, mut data, built)) = panel_query.get_mut(**panel).ok() else {
        return;
    };

    data.localize(&lang);

    built.update(&data, &mut commands);
    stack.push(entity);
    commands.entity(entity).remove::<BodyDataPanelData>();
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
