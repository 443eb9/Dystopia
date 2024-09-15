use bevy::{
    app::{App, Plugin},
    prelude::{Component, Entity, Resource},
};

use crate::{
    body::quantify::{
        AtmosphericDensity, Density, Illuminance, Metallicity, Moisture, Temperature,
    },
    cosmos::celestial::{BodyIndex, BodyTilemap},
};

pub mod quantify;

pub struct DystopiaBodyPlugin;

impl Plugin for DystopiaBodyPlugin {
    fn build(&self, app: &mut App) {}
}

/// The body currently focusing on. Not necessarily exist.
///
/// Inserted by
/// [`handle_body_focusing`](crate::scene::transition::cosmos_view::handle_body_focusing).
#[derive(Resource)]
pub struct FocusingOn {
    pub entity: Entity,
    pub body: BodyIndex,
    pub tilemap: BodyTilemap,
}

#[derive(Component)]
pub struct ParameterizedBody {
    pub temperature: f64,
    pub moisture: f64,
    pub metallicity: f64,
    pub density: f64,
    pub illuminance: f64,
    pub atmospheric_density: f64,
}

impl Default for ParameterizedBody {
    fn default() -> Self {
        Self {
            temperature: f64::NAN,
            moisture: f64::NAN,
            metallicity: f64::NAN,
            density: f64::NAN,
            illuminance: f64::NAN,
            atmospheric_density: f64::NAN,
        }
    }
}

#[derive(Component)]
pub struct QuantifiedBody {
    pub temperature: Temperature,
    pub moisture: Moisture,
    pub metallicity: Metallicity,
    pub density: Density,
    pub illuminance: Illuminance,
    pub atmospheric_density: AtmosphericDensity,
}

impl Default for QuantifiedBody {
    fn default() -> Self {
        Self {
            temperature: Temperature::Cold,
            moisture: Moisture::Dry,
            metallicity: Metallicity::Trace,
            density: Density::Diffuse,
            illuminance: Illuminance::Faint,
            atmospheric_density: AtmosphericDensity::Sparse,
        }
    }
}
