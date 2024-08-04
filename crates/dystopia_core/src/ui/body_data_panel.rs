use bevy::{
    asset::Handle,
    color::Color,
    core::Name,
    log::warn,
    prelude::{
        BuildChildren, ChildBuilder, Component, DetectChanges, Entity, NodeBundle, Query, Res,
        Resource, TextBundle,
    },
    text::{Font, Text, TextStyle},
    ui::{BackgroundColor, BorderColor, FlexDirection, FocusPolicy, Overflow, Style, Val},
};
use dystopia_derive::{AsBuiltComponent, LocalizableEnum, LocalizableStruct};

use crate::{
    cosmos::celestial::{BodyIndex, BodyType, Cosmos, Moon, OrbitIndex, Planet, Star, StarType},
    gen_localizable_enum, key_value_list_element,
    localization::{ui::LUiPanel, LangFile, LocalizableEnumWrapper},
    sci::unit::{Length, Time, Unit},
    ui::{
        common::UiAggregate,
        primitive::AsBuiltUiElement,
        scrollable_list::{ScrollableList, ScrollableListInnerContainer},
    },
};

#[derive(Resource)]
pub struct BodyDataPanel {
    pub target_body: Option<Entity>,
}

gen_localizable_enum!(LBodyType, Star, Planet, Moon);
gen_localizable_enum!(LDetailedBodyType, O, B, A, F, G, K, M, Rocky, Gas, Ice);
gen_localizable_enum!(
    LBodyDataPanelDataField,
    OrbitRadius,
    SiderealPeriod,
    RotationPeriod
);

#[derive(Component, AsBuiltComponent, LocalizableStruct)]
pub struct BodyDataPanelData {
    pub title: LocalizableEnumWrapper<LUiPanel>,
    #[lang_skip]
    pub body_name: String,
    pub body_ty: LocalizableEnumWrapper<LBodyType>,
    pub detailed_body_ty: LocalizableEnumWrapper<LDetailedBodyType>,

    pub orbit_radius: LocalizableEnumWrapper<Length>,
    pub sidereal_period: LocalizableEnumWrapper<Time>,
    pub rotation_period: LocalizableEnumWrapper<Time>,
}

pub struct BodyDataPanelStyle {
    pub width: Val,
    pub height: Val,
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub font: Handle<Font>,
    pub title_font_size: f32,
    pub title_font_color: Color,
    pub title_background_color: Color,
    pub content_font_color: Color,
}

impl UiAggregate for BodyDataPanelData {
    type Style = BodyDataPanelStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        parent
            .spawn(NodeBundle {
                style: Style {
                    width: style.width,
                    height: style.height,
                    ..Default::default()
                },
                background_color: BackgroundColor(style.background_color),
                border_color: BorderColor(style.border_color.unwrap_or(Color::NONE)),
                focus_policy: FocusPolicy::Block,
                ..Default::default()
            })
            .with_children(|root| {
                // Title Bar
                root.spawn(TextBundle {
                    text: Text::from_section(
                        String::default(),
                        TextStyle {
                            font: style.font.clone(),
                            font_size: style.title_font_size,
                            color: style.title_font_color,
                        },
                    ),
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Px(style.title_font_size * 1.5),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(style.title_background_color),
                    ..Default::default()
                });

                // Data
                root.spawn((
                    NodeBundle {
                        style: Style {
                            width: style.width,
                            height: style.height,
                            overflow: Overflow::clip_y(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ScrollableListInnerContainer::default(),
                ))
                .with_children(|list_root| {
                    list_root
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|data_root| {
                            key_value_list_element!(
                                data_root,
                                Val::Percent(20.),
                                TextBundle::from_section(
                                    String::default(),
                                    TextStyle {
                                        font: style.font.clone(),
                                        font_size: 16.,
                                        color: style.content_font_color
                                    }
                                ),
                                TextBundle::from_section(
                                    String::default(),
                                    TextStyle {
                                        font: style.font.clone(),
                                        font_size: 12.,
                                        color: style.content_font_color
                                    }
                                )
                            );
                        });
                });
            })
            .id()
    }
}

pub(super) fn pack_body_data_panel_data(
    panel: Res<BodyDataPanel>,
    body_query: Query<(
        &Name,
        &BodyIndex,
        &OrbitIndex,
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
        orbit_index,
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

    let (Some(body), Some(orbit)) = (
        cosmos.bodies.get(**body_index),
        cosmos.orbits.get(**orbit_index),
    ) else {
        warn!(
            "Failed to get detailed body and orbit data for body {}, named {}",
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
        body_name: body_name.to_string(),
        body_ty,
        detailed_body_ty,
        orbit_radius: Length::wrap_with_si(orbit.radius).into(),
        sidereal_period: Time::wrap_with_si(orbit.sidereal_period).into(),
        rotation_period: Time::wrap_with_si(orbit.rotation_period).into(),
    };
}

pub(super) fn update_ui_panel_data(lang: Res<LangFile>) {}
