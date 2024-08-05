use bevy::{
    app::{App, Plugin, Update},
    core::Name,
    log::warn,
    prelude::{
        in_state, BuildChildren, ChildBuilder, Commands, Component, DetectChanges, Entity,
        IntoSystemConfigs, NodeBundle, Query, Res, ResMut, Resource, TextBundle,
    },
    text::{Text, TextStyle},
    ui::{FlexDirection, FocusPolicy, Style, Val},
};
use dystopia_derive::{AsBuiltComponent, LocalizableEnum, LocalizableStruct};

use crate::{
    cosmos::celestial::{BodyIndex, BodyType, Cosmos, Moon, Planet, Star, StarType},
    distributed_list_element, gen_localizable_enum,
    localization::{ui::LUiPanel, LangFile, LocalizableDataWrapper, LocalizableStruct},
    merge_list,
    schedule::state::GameState,
    sci::unit::{Length, Time, Unit},
    ui::{
        common::UiAggregate,
        ext::DefaultWithStyle,
        preset::{
            default_panel_style, default_section_style, default_title_style, PANEL_BORDER_COLOR,
            PANEL_BACKGROUND, PANEL_ELEM_TEXT_STYLE, PANEL_SUBTITLE_TEXT_STYLE,
            PANEL_TITLE_BACKGROUND, PANEL_TITLE_FONT_SIZE, PANEL_TITLE_TEXT_COLOR,
        },
        primitive::AsBuiltComponent,
        scrollable_list::ScrollableList,
        GlobalUiRoot, UiBuilder, FUSION_PIXEL,
    },
};

#[derive(Resource, Default)]
pub struct BodyDataPanel {
    pub panel: Option<Entity>,
    pub target_body: Option<Entity>,
}

gen_localizable_enum!(LBodyType, Star, Planet, Moon);
gen_localizable_enum!(LDetailedBodyType, O, B, A, F, G, K, M, Rocky, Gas, Ice);
gen_localizable_enum!(
    LBodyOrbitInfoType,
    OrbitRadius,
    SiderealPeriod,
    RotationPeriod
);
gen_localizable_enum!(LBodyDataPanelSectionType, BodyInfo, OrbitInfo);

pub struct BodyDataPanelPlugin;

impl Plugin for BodyDataPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pack_body_data_panel_data, update_ui_panel_data).run_if(in_state(GameState::Simulate)),
        )
        .init_resource::<BodyDataPanel>();
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
    title_orbit_radius: LocalizableDataWrapper<LBodyOrbitInfoType>,
    orbit_radius: LocalizableDataWrapper<Length>,
    title_sidereal_period: LocalizableDataWrapper<LBodyOrbitInfoType>,
    sidereal_period: LocalizableDataWrapper<Time>,
    title_rotation_period: LocalizableDataWrapper<LBodyOrbitInfoType>,
    rotation_period: LocalizableDataWrapper<Time>,
}

pub struct BodyDataPanelStyle {
    pub width: Val,
    pub height: Val,
}

impl UiAggregate for BodyDataPanelData {
    type Style = BodyDataPanelStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut entities = Vec::with_capacity(Self::NUM_FIELDS);

        parent
            .spawn((
                NodeBundle {
                    style: Style {
                        width: style.width,
                        height: style.height,
                        ..default_panel_style()
                    },
                    background_color: PANEL_BACKGROUND,
                    border_color: PANEL_BORDER_COLOR.into(),
                    focus_policy: FocusPolicy::Block,
                    ..Default::default()
                },
                Name::new("BodyDataPanel"),
            ))
            .with_children(|root| {
                // Title Bar
                entities.push(
                    root.spawn(TextBundle {
                        text: Text::default_with_style(TextStyle {
                            font: FUSION_PIXEL,
                            font_size: PANEL_TITLE_FONT_SIZE,
                            color: PANEL_TITLE_TEXT_COLOR,
                        }),
                        style: default_title_style(),
                        background_color: PANEL_TITLE_BACKGROUND,
                        ..Default::default()
                    })
                    .id(),
                );

                // Data
                root.spawn((
                    NodeBundle {
                        style: Style {
                            width: style.width,
                            height: style.height,
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
                                    .spawn(TextBundle::default_with_style(
                                        PANEL_SUBTITLE_TEXT_STYLE,
                                    ))
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
                                    .spawn(TextBundle::default_with_style(
                                        PANEL_SUBTITLE_TEXT_STYLE,
                                    ))
                                    .id(),
                            );

                            entities.extend(merge_list!(
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

pub(super) fn pack_body_data_panel_data(
    mut commands: Commands,
    mut panel: ResMut<BodyDataPanel>,
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
) {
    if !panel.is_changed() {
        return;
    }

    let Some(target_body) = panel.target_body else {
        return;
    };

    let Ok((
        body_name,
        body_index,
        maybe_star,
        maybe_star_ty,
        maybe_planet,
        maybe_moon,
        maybe_body_ty,
    )) = body_query.get(target_body)
    else {
        warn!("Failed to find the target body.");
        return;
    };

    // body indices should be equal to orbit indices
    let Some(orbit) = cosmos.orbits.get(**body_index) else {
        warn!(
            "Failed to get orbit data for body {}, named {}",
            **body_index,
            body_name.as_str()
        );
        return;
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

    let data = BodyDataPanelData {
        title: LUiPanel::BodyData.into(),
        section_body_info: LBodyDataPanelSectionType::BodyInfo.into(),
        body_name: body_name.to_string(),
        body_ty,
        detailed_body_ty,
        section_orbit_info: LBodyDataPanelSectionType::OrbitInfo.into(),
        title_orbit_radius: LBodyOrbitInfoType::OrbitRadius.into(),
        orbit_radius: Length::wrap_with_si(orbit.radius).into(),
        title_sidereal_period: LBodyOrbitInfoType::SiderealPeriod.into(),
        sidereal_period: Time::wrap_with_si(orbit.sidereal_period).into(),
        title_rotation_period: LBodyOrbitInfoType::RotationPeriod.into(),
        rotation_period: Time::wrap_with_si(orbit.rotation_period).into(),
    };

    if let Some(panel) = panel.panel {
        commands.entity(panel).insert(data);
    } else {
        panel.panel = Some(commands.spawn(data).id());
    }
}

fn update_ui_panel_data(
    mut commands: Commands,
    lang: Res<LangFile>,
    mut panel: ResMut<BodyDataPanel>,
    mut panel_query: Query<(
        Entity,
        &mut BodyDataPanelData,
        Option<&BuiltBodyDataPanelData>,
    )>,
    global_root: Res<GlobalUiRoot>,
) {
    if panel_query.is_empty() {
        return;
    }

    let Some((entity, mut data, built)) = panel.panel.and_then(|e| panel_query.get_mut(e).ok())
    else {
        return;
    };

    data.localize(&lang);

    if let Some(built) = built {
        built.update(&data, &mut commands);
        commands.entity(entity).remove::<BodyDataPanelData>();
    } else {
        let mut built = None;
        commands.entity(**global_root).with_children(|root| {
            built = Some(root.build_ui(
                &*data,
                BodyDataPanelStyle {
                    width: Val::Px(250.),
                    height: Val::Px(500.),
                },
            ));
        });
        panel.panel = built;
        commands.entity(entity).despawn();
    }
}
