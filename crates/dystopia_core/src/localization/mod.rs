use bevy::{
    app::{App, Plugin},
    asset::Asset,
    prelude::{Deref, Entity, Resource},
    reflect::TypePath,
    utils::HashMap,
};
use serde::Deserialize;

use crate::{
    assets::{app_ext::DystopiaAssetAppExt, config::RawConfig},
    ui::primitive::AsBuiltUiElement,
};

pub mod macros;
pub mod time;
pub mod ui;

pub struct DystopiaLocalizationPlugin;

impl Plugin for DystopiaLocalizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_config::<LangFile>();
    }
}

/// The localization file.
///
/// `HashMap<EnumName, HashMap<VariantName, LocalizedName>`
#[derive(Resource, Asset, TypePath, Clone, Deserialize, Deref)]
pub struct LangFile(HashMap<String, HashMap<String, String>>);

impl RawConfig for LangFile {
    type Processed = Self;

    // TODO Language switching
    const NAME: &'static str = "localization/current.json";
}

/// Before data being passed to UI components, it should be localized.
///
/// For each field in [`LocalizableStruct`], you should label all possible values on that,
/// unless it has attribute `#[lang_skip]`.
pub trait LocalizableStruct {
    fn localize(&mut self, lang: &LangFile);
}

/// [`LocalizableEnum`]s can be localized without knowing the struct or field it belongs to.
pub trait LocalizableEnum {
    fn localize(&self, lang: &LangFile) -> String;
}

pub enum LocalizableEnumWrapper<E: LocalizableEnum> {
    Raw(E),
    Localized(String),
}

impl<E: LocalizableEnum> AsBuiltUiElement for LocalizableEnumWrapper<E> {
    type BuiltType = Entity;
}

impl<E: LocalizableEnum> From<E> for LocalizableEnumWrapper<E> {
    fn from(value: E) -> Self {
        Self::Raw(value)
    }
}

impl<E: LocalizableEnum> LocalizableEnumWrapper<E> {
    #[inline]
    pub fn localize(&mut self, lang: &LangFile) {
        let s = match &*self {
            LocalizableEnumWrapper::Raw(r) => &r.localize(lang),
            LocalizableEnumWrapper::Localized(l) => l,
        };
        *self = Self::Localized(s.to_owned());
    }

    #[inline]
    pub fn localized(&self) -> String {
        match self {
            LocalizableEnumWrapper::Raw(_) => panic!("This data is not localized yet."),
            LocalizableEnumWrapper::Localized(l) => l.to_owned(),
        }
    }
}
