use bevy::{
    core::Name,
    log::warn,
    prelude::{Component, DetectChanges, Entity, Query, Res, Resource},
};
use dystopia_derive::{AsBuiltComponent, LocalizableEnum, LocalizableStruct};

use crate::{
    cosmos::celestial::{BodyIndex, BodyType, Cosmos, Moon, OrbitIndex, Planet, Star, StarType},
    gen_localizable_enum,
    localization::LocalizableEnumWrapper,
    sci::unit::{Length, Time},
    ui::primitive::AsBuiltUiElement,
};

#[derive(Resource)]
pub struct BodyDataPanel {
    pub target_body: Option<Entity>,
}

gen_localizable_enum!(LBodyType, Star, Planet, Moon);
gen_localizable_enum!(LDetailedBodyType, O, B, A, F, G, K, M, Rocky, Gas, Ice);

#[derive(Component, AsBuiltComponent, LocalizableStruct)]
pub(super) struct BodyDataPanelData {
    #[lang_skip]
    pub body_name: String,
    pub body_ty: LocalizableEnumWrapper<LBodyType>,
    pub detailed_body_ty: LocalizableEnumWrapper<LDetailedBodyType>,

    pub orbit_radius: LocalizableEnumWrapper<Length>,
    pub sidereal_period: LocalizableEnumWrapper<Time>,
    pub rotation_period: LocalizableEnumWrapper<Time>,
}

pub fn pack_body_data_panel_data(
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
}
