use bevy::{
    app::{App, Plugin},
    asset::Asset,
    prelude::{Deref, Resource},
    reflect::TypePath,
    text::Text,
    utils::HashMap,
};
use serde::Deserialize;

use crate::{
    assets::{app_ext::DystopiaAssetAppExt, config::RawConfig},
    ui::update::{AsOriginalComponent, AsUpdatableData},
};

pub mod macros;
pub mod number;
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
    const PATH: &'static str = "localization/current.json";
}

/// Before data being passed to UI components, it should be localized.
///
/// For each field in [`LocalizableStruct`], you should label all possible values on that,
/// unless it has attribute `#[lang_skip]`.
pub trait LocalizableStruct {
    fn localize(&mut self, lang: &LangFile);
}

/// [`LocalizableData`]s can be localized without knowing the struct or field it belongs to.
pub trait LocalizableData {
    fn localize(&self, lang: &LangFile) -> String;
}

#[derive(Clone)]
pub enum LocalizableDataWrapper<E: LocalizableData> {
    Raw(E),
    Localized(String),
}

impl<E: LocalizableData> From<E> for LocalizableDataWrapper<E> {
    fn from(value: E) -> Self {
        Self::Raw(value)
    }
}

impl<E: LocalizableData> From<LocalizableDataWrapper<E>> for String {
    fn from(value: LocalizableDataWrapper<E>) -> Self {
        value.localized()
    }
}

impl<E: LocalizableData> AsOriginalComponent for LocalizableDataWrapper<E> {
    type OriginalComponent = Text;
}

impl<E: LocalizableData> AsUpdatableData for LocalizableDataWrapper<E> {
    type UpdatableData = String;
}

impl<E: LocalizableData + Default> Default for LocalizableDataWrapper<E> {
    fn default() -> Self {
        Self::Raw(E::default())
    }
}

impl<E: LocalizableData> LocalizableDataWrapper<E> {
    #[inline]
    pub fn localize(&mut self, lang: &LangFile) {
        let s = match &*self {
            LocalizableDataWrapper::Raw(r) => &r.localize(lang),
            LocalizableDataWrapper::Localized(l) => l,
        };
        *self = Self::Localized(s.to_owned());
    }

    #[inline]
    pub fn localized(&self) -> String {
        match self {
            LocalizableDataWrapper::Raw(_) => panic!("This data has not localized yet."),
            LocalizableDataWrapper::Localized(l) => l.to_owned(),
        }
    }
}
