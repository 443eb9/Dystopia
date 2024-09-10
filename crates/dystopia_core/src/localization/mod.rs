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
/// For each field in [`LocalizableData`], you should label all possible values on that,
/// unless it has attribute `#[lang_skip]`.
pub trait LocalizableData {
    fn localize(&mut self, lang: &LangFile);
}

impl<T: LocalizableData> LocalizableData for Vec<T> {
    fn localize(&mut self, lang: &LangFile) {
        self.iter_mut().for_each(|e| e.localize(lang));
    }
}

/// [`LocalizablePrimitive`]s can be localized without knowing the struct or field it belongs to.
pub trait LocalizablePrimitive {
    fn localize(&self, lang: &LangFile) -> String;
}

#[derive(Clone)]
pub enum Localizable<E: LocalizablePrimitive> {
    Raw(E),
    Localized(String),
}

impl<E: LocalizablePrimitive> From<E> for Localizable<E> {
    fn from(value: E) -> Self {
        Self::Raw(value)
    }
}

impl<E: LocalizablePrimitive> From<Localizable<E>> for String {
    fn from(value: Localizable<E>) -> Self {
        value.localized()
    }
}

impl<E: LocalizablePrimitive> AsOriginalComponent for Localizable<E> {
    type OriginalComponent = Text;
}

impl<E: LocalizablePrimitive> AsUpdatableData for Localizable<E> {
    type UpdatableData = String;
}

impl<E: LocalizablePrimitive + Default> Default for Localizable<E> {
    fn default() -> Self {
        Self::Raw(E::default())
    }
}

impl<E: LocalizablePrimitive> Localizable<E> {
    #[inline]
    pub fn localize(&mut self, lang: &LangFile) {
        let s = match &*self {
            Localizable::Raw(r) => &r.localize(lang),
            Localizable::Localized(l) => l,
        };
        *self = Self::Localized(s.to_owned());
    }

    #[inline]
    pub fn localized(&self) -> String {
        match self {
            Localizable::Raw(_) => panic!("This data has not localized yet."),
            Localizable::Localized(l) => l.to_owned(),
        }
    }
}
