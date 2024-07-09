use bevy::{
    asset::Asset,
    color::{Color, LinearRgba, Srgba},
    prelude::Resource,
    reflect::TypePath,
};
use serde::{Deserialize, Serialize};

use crate::{assets::config::RawConfig, cosmos::celestial::StarType};

#[derive(Resource)]
pub struct CosmosStarPropertiesConfig {
    config: Vec<StarProperties>,
}

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub(super) struct RawCosmosStarPropertiesConfig(Vec<RawStarProperties>);

impl RawConfig<CosmosStarPropertiesConfig> for RawCosmosStarPropertiesConfig {
    const NAME: &'static str = "star_properties.json";

    fn process(&self) -> CosmosStarPropertiesConfig {
        CosmosStarPropertiesConfig {
            config: self.0.clone().into_iter().map(Into::into).collect(),
        }
    }
}

/// Detailed class of a main sequence star.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StarClass {
    pub ty: StarType,
    pub sub_ty: u32,
}

/// Basic properties of a star defined in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStarProperties {
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
    /// In sum luminosity
    pub luminosity: f64,
    /// In kelvin.
    pub effective_temp: f64,
    pub color: LinearRgba,
}
