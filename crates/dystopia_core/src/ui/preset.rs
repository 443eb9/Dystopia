use bevy::{
    color::{
        palettes::css::{BLACK, WHITE},
        Color, Srgba,
    },
    text::TextStyle,
    ui::{BackgroundColor, FlexDirection, PositionType, Style, UiRect, Val},
};

use crate::ui::FUSION_PIXEL;

pub fn default_panel_style() -> Style {
    Style {
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        border: UiRect::all(Val::Px(2.)),
        ..Default::default()
    }
}

pub fn default_title_style() -> Style {
    Style {
        width: Val::Percent(100.),
        height: Val::Px(PANEL_TITLE_HEIGHT),
        ..Default::default()
    }
}

pub fn default_section_style() -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        border: UiRect::all(Val::Px(2.)),
        margin: UiRect::all(Val::Px(SECTION_MARGIN)),
        ..Default::default()
    }
}

pub const SECTION_MARGIN: f32 = 7.;

pub const PANEL_BACKGROUND: BackgroundColor = BackgroundColor(Color::WHITE);

pub const PANEL_TITLE_TEXT_STYLE: TextStyle = TextStyle {
    font: FUSION_PIXEL,
    font_size: 20.,
    color: Color::Srgba(WHITE),
};
pub const PANEL_TITLE_TEXT_COLOR: Color = Color::WHITE;
pub const PANEL_TITLE_FONT_SIZE: f32 = 20.;
pub const PANEL_TITLE_BACKGROUND: BackgroundColor = BackgroundColor(PANEL_BORDER_COLOR);
pub const PANEL_TITLE_HEIGHT: f32 = 25.;

pub const PANEL_SUBTITLE_TEXT_STYLE: TextStyle = TextStyle {
    font: FUSION_PIXEL,
    font_size: 20.,
    color: Color::Srgba(BLACK),
};
pub const PANEL_SUBTITLE_TEXT_COLOR: Color = PANEL_BORDER_COLOR;
pub const PANEL_SUBTITLE_FONT_SIZE: f32 = 18.;

pub const PANEL_ELEM_TEXT_STYLE: TextStyle = TextStyle {
    font: FUSION_PIXEL,
    font_size: 12.,
    color: Color::Srgba(BLACK),
};
pub const PANEL_ELEM_TEXT_COLOR: Color = PANEL_BORDER_COLOR;
pub const PANEL_ELEM_FONT_SIZE: f32 = 14.;

pub const PANEL_BORDER_COLOR: Color = Color::Srgba(Srgba::rgb(68. / 255., 62. / 255., 185. / 255.));
pub const SECTION_BORDER_COLOR: Color = Color::Srgba(Srgba::new(0., 0., 0., 0.2));

pub const FULLSCREEN_UI_CORNERS: UiRect = UiRect::all(Val::Px(10.));
