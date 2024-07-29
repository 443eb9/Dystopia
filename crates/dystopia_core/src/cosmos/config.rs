use bevy::{
    asset::Asset,
    color::{Color, LinearRgba, Srgba},
    prelude::{Deref, Resource},
    reflect::TypePath,
};
use serde::{Deserialize, Serialize};

use crate::{assets::config::RawConfig, cosmos::celestial::StarClass};

/// All properties about a main sequence star. This vector is sorted descending by
/// the mass of bodies.
#[derive(Resource, Deref)]
pub struct CosmosStarPropertiesConfig(Vec<StarProperties>);

/// All possible names of a star.
#[derive(Resource, Asset, TypePath, Clone, Deserialize, Deref)]
pub struct CosmosStarNamesConfig(Vec<String>);

impl RawConfig for CosmosStarNamesConfig {
    type Processed = Self;

    const NAME: &'static str = "star_names.json";
}

#[derive(Asset, TypePath, Clone, Serialize, Deserialize)]
pub(super) struct RawCosmosStarPropertiesConfig(Vec<RawStarProperties>);

impl RawConfig for RawCosmosStarPropertiesConfig {
    type Processed = CosmosStarPropertiesConfig;

    const NAME: &'static str = "star_properties.json";
}

impl From<RawCosmosStarPropertiesConfig> for CosmosStarPropertiesConfig {
    fn from(value: RawCosmosStarPropertiesConfig) -> Self {
        CosmosStarPropertiesConfig(value.0.into_iter().map(Into::into).collect())
    }
}

/// Basic properties of a star defined in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawStarProperties {
    pub class: StarClass,
    pub mass: f64,
    pub radius: f64,
    pub luminosity: f64,
    pub effective_temp: f64,
    pub color: String,
}

impl Into<StarProperties> for RawStarProperties {
    fn into(self) -> StarProperties {
        StarProperties {
            class: self.class,
            mass: self.mass,
            radius: self.radius,
            luminosity: self.luminosity,
            effective_temp: self.effective_temp,
            color: Color::Srgba(Srgba::hex(self.color).unwrap()).to_linear(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StarProperties {
    pub class: StarClass,
    /// In sun mass.
    pub mass: f64,
    /// In sun radius.
    pub radius: f64,
    /// In sun luminosity.
    pub luminosity: f64,
    /// In kelvin.
    pub effective_temp: f64,
    pub color: LinearRgba,
}
