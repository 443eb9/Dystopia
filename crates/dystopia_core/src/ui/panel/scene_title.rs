use bevy::{
    app::{App, Plugin, Update},
    color::LinearRgba,
    core::Name,
    prelude::{
        in_state, resource_exists, BuildChildren, ChildBuilder, Commands, Component, Deref,
        DerefMut, Entity, EventReader, IntoSystemConfigs, NodeBundle, Query, Res, ResMut, Resource,
        TextBundle,
    },
    text::{JustifyText, Text, TextSection, TextStyle},
    ui::{FlexDirection, Style, Val},
};
use dystopia_derive::{AsBuiltComponent, LocalizableStruct};

use crate::{
    input::RayTransparent,
    localizable_enum,
    localization::{LangFile, LocalizableDataWrapper, LocalizableStruct},
    schedule::state::GameState,
    ui::{
        panel::PanelTargetChange,
        transition::UiTransition,
        update::{AsBuiltComponent, UiDataUpdate},
        GlobalUiRoot, UiAggregate, UiBuilder, FUSION_PIXEL,
    },
    util::alpha::Alpha,
};

localizable_enum!(LSceneTitle, pub, CosmosView, FocusingBody);

pub struct SceneTitlePlugin;

impl Plugin for SceneTitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PanelTargetChange<SceneTitle, SceneTitleChange>>()
            .add_systems(
                Update,
                (
                    pack_scene_title_data,
                    update_scene_title.run_if(resource_exists::<SceneTitle>),
                )
                    .run_if(in_state(GameState::Simulate)),
            );
    }
}

pub const TITLE_FADE_DURATION: f32 = 4.;
pub const TITLE_FADE_DEFER: f32 = 1.5;
pub const TITLE_FONT_SIZE: f32 = 48.;

#[derive(Resource, Deref, DerefMut)]
pub struct SceneTitle(Entity);

#[derive(Component, AsBuiltComponent, LocalizableStruct)]
pub struct SceneTitleData {
    title: LocalizableDataWrapper<LSceneTitle>,
    #[lang_skip]
    #[share_entity(1)]
    name_color: LinearRgba,
    #[lang_skip]
    name: String,
}

pub struct SceneTitleStyle {
    pub font_size: f32,
    pub color: LinearRgba,
}

impl UiAggregate for SceneTitleData {
    type Style = SceneTitleStyle;

    fn build(&self, parent: &mut ChildBuilder, style: Self::Style) -> Entity {
        let mut entities = Vec::with_capacity(Self::NUM_FIELDS);

        let mut root = parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    top: Val::Px(5.),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("SceneTitle"),
            RayTransparent,
        ));

        root.with_children(|root| {
            // Title
            entities.push(
                root.spawn((
                    TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: Default::default(),
                                style: TextStyle {
                                    font: FUSION_PIXEL,
                                    font_size: style.font_size,
                                    color: style.color.into(),
                                },
                            }],
                            justify: JustifyText::Center,
                            ..Default::default()
                        },
                        style: Style {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    RayTransparent,
                ))
                .id(),
            );

            // Name
            entities.push(
                root.spawn((
                    TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: Default::default(),
                                style: TextStyle {
                                    font: FUSION_PIXEL,
                                    font_size: style.font_size,
                                    color: style.color.into(),
                                },
                            }],
                            justify: JustifyText::Center,
                            ..Default::default()
                        },
                        style: Style {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    RayTransparent,
                ))
                .id(),
            );
        })
        .insert(BuiltSceneTitleData::from_entities(entities))
        .id()
    }
}

pub struct SceneTitleChange {
    pub title: LSceneTitle,
    pub name: Option<(String, LinearRgba)>,
}

fn pack_scene_title_data(
    mut commands: Commands,
    mut target_change: EventReader<PanelTargetChange<SceneTitle, SceneTitleChange>>,
    global_root: Res<GlobalUiRoot>,
    panel: Option<ResMut<SceneTitle>>,
) {
    for target in target_change.read() {
        let SceneTitleChange { title, name } = target.as_ref().unwrap();
        let (name, name_color) = name.clone().unwrap_or_default();

        let data = SceneTitleData {
            title: title.clone().into(),
            name,
            name_color,
        };

        if let Some(built) = panel.as_deref() {
            commands.entity(**built).insert(data);
        } else {
            let mut built = None;
            commands.entity(**global_root).with_children(|root| {
                built = Some(root.build_ui(
                    &data,
                    SceneTitleStyle {
                        font_size: TITLE_FONT_SIZE,
                        color: LinearRgba::WHITE,
                    },
                ));
            });

            let panel = built.unwrap();
            commands.insert_resource(SceneTitle(panel));
            commands.entity(panel).insert(data);
        }
    }
}

fn update_scene_title(
    mut commands: Commands,
    lang: Res<LangFile>,
    panel: Res<SceneTitle>,
    mut panel_query: Query<(&mut SceneTitleData, &BuiltSceneTitleData)>,
) {
    let panel = **panel;
    let Ok((mut data, built)) = panel_query.get_mut(panel) else {
        return;
    };

    data.localize(&lang);
    built.update(&data, &mut commands);

    commands.entity(panel).remove::<SceneTitleData>();

    let trans = UiTransition::<Text, _>::new(Alpha::new(0.), TITLE_FADE_DURATION, TITLE_FADE_DEFER);
    commands
        .entity(built.name)
        .insert(trans.clone())
        .insert(UiDataUpdate::<Text, _>::new(Alpha::new(1.)));
    commands
        .entity(built.title)
        .insert(trans)
        .insert(UiDataUpdate::<Text, _>::new(Alpha::new(1.)));
}
