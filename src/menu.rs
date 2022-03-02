use crate::common::{CurrentDay, EnemyMorale, GameFonts, GameState, OpeningNarration, Ui};
use bevy::prelude::*;

// Narration

const NARRATION_LENGTH: usize = 2;
const OPENING_NARRATION: [&str; NARRATION_LENGTH] = [
    "You are a lich, powerful and unmatched.
\nYou have ruled over this world for a long, long time.
The armies that attempt to destroy you
only feed you with more souls.
\nBut it is a delicate balance.",
    "If you utterly destroy them too many times, they will lose hope.
The attacks that feed you will stop,
and one day, you will run out of souls.
\nBut if you show your weakness too much, they will find their courage.
They will stop seeing you as undefeatable, and they will win.
\nYou must let them hope - but never let them stop fearing.",
];

const BUTTON_NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
const BUTTON_HOVER: Color = Color::rgb(0.25, 0.25, 0.25);
const BUTTON_PRESSED: Color = Color::rgb(0.75, 0.75, 0.35);

const TEXT_COLOR: Color = Color::rgb(0.85, 0.85, 0.85);

#[allow(clippy::type_complexity)]
pub fn button_shift_narration(
    mut q_interaction: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
    mut q_narration_text: Query<(&mut Text, &mut OpeningNarration)>,
    mut state: ResMut<State<GameState>>,
) {
    if let Some((mut narration_text, mut narration_pos)) = q_narration_text.iter_mut().next() {
        for (interaction, mut color) in q_interaction.iter_mut() {
            match *interaction {
                Interaction::Clicked => {
                    narration_pos.0 += 1;
                    if narration_pos.0 >= NARRATION_LENGTH {
                        state.set(GameState::MoraleStatus).unwrap();
                    } else {
                        narration_text.sections[0].value =
                            OPENING_NARRATION[narration_pos.0].to_string();
                    }
                    *color = BUTTON_PRESSED.into();
                }
                Interaction::Hovered => {
                    *color = BUTTON_HOVER.into();
                }
                Interaction::None => {
                    *color = BUTTON_NORMAL.into();
                }
            }
        }
    }
}

pub fn spawn_menu(mut commands: Commands, fonts: Res<GameFonts>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(Ui::Core)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        OPENING_NARRATION[0],
                        TextStyle {
                            font: fonts.main.clone(),
                            font_size: 32.0,
                            color: TEXT_COLOR,
                        },
                        TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            vertical: VerticalAlign::Center,
                        },
                    ),
                    ..Default::default()
                })
                .insert(Ui::NarrationText)
                .insert(OpeningNarration(0));

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        margin: Rect {
                            top: Val::Px(30.0),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: BUTTON_NORMAL.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Next",
                            TextStyle {
                                font: fonts.main.clone(),
                                font_size: 32.0,
                                color: TEXT_COLOR,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

// Morale status

#[allow(clippy::type_complexity)]
pub fn button_start_day(
    mut q_interaction: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<State<GameState>>,
    mut current_day: ResMut<CurrentDay>,
) {
    for (interaction, mut color) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                current_day.0 += 1;
                *color = BUTTON_PRESSED.into();
                state.set(GameState::ActiveGame).unwrap();
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_NORMAL.into();
            }
        }
    }
}

pub fn spawn_morale_status(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    morale: Res<EnemyMorale>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(Ui::Core)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Humanity's morale is currently at:\n".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: format!("{:.1}%\n", morale.0),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 64.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "The next army is about to arrive...".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                        ],
                        alignment: TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            vertical: VerticalAlign::Center,
                        },
                    },
                    ..Default::default()
                })
                .insert(Ui::NarrationText);

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        margin: Rect {
                            top: Val::Px(30.0),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: BUTTON_NORMAL.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Start Day",
                            TextStyle {
                                font: fonts.main.clone(),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

pub fn despawn_menu(mut commands: Commands, q_ui: Query<(Entity, &Ui), With<Children>>) {
    for (ent, ui) in q_ui.iter() {
        if let Ui::Core = ui {
            commands.entity(ent).despawn_recursive();
        }
    }
}
