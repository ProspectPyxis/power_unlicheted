use crate::common::{
    CurrentDay, DayEndReason, EndDayEvent, EnemyMorale, GameAudio, GameFonts, GameOverButton,
    GameSprites, GameState, MainMenuButton, NarrationViewed, OpeningNarration, Ui,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_kira_audio::Audio;

const NARRATION_LENGTH: usize = 2;
const OPENING_NARRATION: [&str; NARRATION_LENGTH] = [
    "You are a lich, powerful and unmatched.
\nYou have ruled over this world for a long, long time.
The armies that attempt to destroy you
only feed you with more souls.
\nBut it is a delicate balance.",
    "If you utterly destroy them too many times,
they will lose hope.
The attacks that feed you will stop,
and one day, you will run out of souls.
\nBut if you show your weakness too much,
they will find their courage.
They will stop seeing you as undefeatable,
and they will win.
\nYou must let them hope - but never let them stop fearing.",
];

const GAME_TIPS_COUNT: usize = 6;
const GAME_TIPS: [&str; GAME_TIPS_COUNT] = [
    "Your body can be killed, as long as your phylactery lives.
Perhaps letting them \"kill\" you can give them some hope.",
    "Letting the soldiers flee with their lives can make them believe you're weak.
Maybe one of your spells can assist with this...",
    "They want to believe they are making progress in destroying you.
Showing you cannot be hurt by them could crush their morale.",
    "They want to believe they are making progress in destroying you.
If you want to give them hope, let them hurt you.",
    "Humanity is smart - they will catch on to your intents sooner or later.
Morale will become harder to keep in check over time.",
    "You need to put up at least somewhat of a fight.
Hold back too much, and they will realize your manipulations.",
];

const BUTTON_NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
const BUTTON_HOVER: Color = Color::rgb(0.25, 0.25, 0.25);

const TEXT_COLOR: Color = Color::rgb(0.85, 0.85, 0.85);

// Main Menu

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn button_main_menu(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &MainMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
    mut morale: ResMut<EnemyMorale>,
    mut current_day: ResMut<CurrentDay>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
    mut narration_viewed: ResMut<NarrationViewed>,
) {
    for (interaction, mut color, button_type) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                audio_player.play(audio.click.clone());
                match *button_type {
                    MainMenuButton::Start => {
                        current_day.day = 0;
                        morale.current = 50.0;
                        if !narration_viewed.0 {
                            narration_viewed.0 = true;
                            state.set(GameState::Opening).unwrap();
                        } else {
                            state.set(GameState::MoraleStatus).unwrap();
                        }
                    }
                    MainMenuButton::Credits => {
                        state.set(GameState::Credits).unwrap();
                    }
                }
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

pub fn spawn_main_menu(mut commands: Commands, fonts: Res<GameFonts>, sprites: Res<GameSprites>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(Ui::Core)
        .with_children(|parent| {
            parent.spawn_bundle(ImageBundle {
                image: UiImage(sprites.game_logo.clone()),
                ..Default::default()
            });

            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    let spawn_button =
                        |parent: &mut ChildBuilder, text: &str, context: MainMenuButton| {
                            parent
                                .spawn_bundle(ButtonBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                        margin: Rect {
                                            top: Val::Px(30.0),
                                            left: Val::Px(30.0),
                                            right: Val::Px(30.0),
                                            ..Default::default()
                                        },
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..Default::default()
                                    },
                                    color: BUTTON_NORMAL.into(),
                                    ..Default::default()
                                })
                                .insert(context)
                                .with_children(|parent| {
                                    parent.spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            text.to_string(),
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
                        };

                    spawn_button(parent, "Start", MainMenuButton::Start);
                    spawn_button(parent, "Credits", MainMenuButton::Credits);
                });
        });
}

// Narration

#[allow(clippy::type_complexity)]
pub fn button_shift_narration(
    mut q_interaction: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
    mut q_narration_text: Query<(&mut Text, &mut OpeningNarration)>,
    mut state: ResMut<State<GameState>>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
) {
    if let Some((mut narration_text, mut narration_pos)) = q_narration_text.iter_mut().next() {
        for (interaction, mut color) in q_interaction.iter_mut() {
            match *interaction {
                Interaction::Clicked => {
                    audio_player.play(audio.click.clone());
                    narration_pos.0 += 1;
                    if narration_pos.0 >= NARRATION_LENGTH {
                        state.set(GameState::MoraleStatus).unwrap();
                    } else {
                        narration_text.sections[0].value =
                            OPENING_NARRATION[narration_pos.0].to_string();
                    }
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
                                color: Color::WHITE,
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
    morale: Res<EnemyMorale>,
    mut current_day: ResMut<CurrentDay>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
) {
    for (interaction, mut color) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                audio_player.play(audio.click.clone());
                if morale.current == 0.0 || morale.current == 100.0 {
                    state.set(GameState::GameOver).unwrap();
                } else {
                    current_day.player_damaged = 0.0;
                    current_day.day += 1;
                    state.set(GameState::ActiveGame).unwrap();
                }
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
    mut morale: ResMut<EnemyMorale>,
    current_day: Res<CurrentDay>,
    mut day_end_reader: EventReader<EndDayEvent>,
) {
    let day_end = day_end_reader.iter().next();

    // Setup morale values
    if let Some(day_end) = day_end {
        if morale.enemies_killed < 40 {
            morale.change = (morale.change - 15.0).min(-15.0);
        } else if let DayEndReason::PlayerDeath = day_end.reason {
            morale.change = (morale.change + 25.0).max(10.0);
        } else if current_day.player_damaged < 3.0 {
            morale.change -= 10.0;
        } else {
            morale.change += current_day.player_damaged / 10.0;
        }
    }
    if current_day.day == 1 {
        morale.current = (morale.current + morale.change).clamp(15.0, 85.0);
    } else if current_day.day > 0 {
        morale.current =
            (morale.current + morale.change + (morale.change * (current_day.day - 1) as f32 / 4.0))
                .clamp(0.0, 100.0);
    }

    let morale_text_prelude = if let Some(day_end) = day_end {
        match day_end.reason {
            DayEndReason::Timeout => {
                if morale.enemies_killed < 40 {
                    "As the soldiers' adrenaline fades,
they realize they barely lost anybody.
Considering your immense power,
this turn of events greatly confuses them.\n\n"
                } else if current_day.player_damaged < 3.0 {
                    "You have shown them your strength today - 
the army barely hurt you at all.
They despair at their powerlessness.\n\n"
                } else {
                    "As the day closes, you take stock
of your action's effects.\n\n"
                }
            }
            DayEndReason::PlayerDeath => {
                if morale.enemies_killed < 40 {
                    "They have defeated you easily - too easily.
The army quickly catches onto your feint,
and they become more hesitant to attack.\n\n"
                } else {
                    "Whether by carelessness or intentional feint,
you have fallen in battle today.
Your phylactery keeps you alive,
but the army celebrates its victory.\n\n"
                }
            }
        }
    } else {
        ""
    };

    let morale_text_end = if current_day.day == 0 {
        "\nThe next army is about to arrive..."
    } else if morale.current == 0.0 || morale.current == 100.0 {
        ""
    } else if morale.current >= 75.0 {
        "\nThey are beginning to grow brave.
Perhaps you should be harsher on them
and show them their place."
    } else if morale.current <= 25.0 {
        "\nThey grow hopeless by the day.
Perhaps you should show them mercy - 
allow them to hurt you, or escape your wrath."
    } else {
        "\nThey have hope, but they still fear.
You should maintain this balance."
    };

    let button_text = if current_day.day == 0 {
        "Start Day"
    } else if morale.current == 0.0 || morale.current == 100.0 {
        "Game Over"
    } else {
        "Next Day"
    };

    let game_tip = if morale.current == 0.0 || morale.current == 100.0 {
        "".to_string()
    } else {
        format!(
            "\n\nTip: {}",
            GAME_TIPS[alea::u32_less_than(GAME_TIPS_COUNT as u32) as usize]
        )
    };

    morale.change = 0.0;
    morale.enemies_killed = 0;

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
                                value: morale_text_prelude.to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "Humanity's morale is currently at:".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: format!("\n{:.1}%", morale.current),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 64.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: morale_text_end.to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: game_tip,
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 16.0,
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
                            button_text,
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

// Game over

#[allow(clippy::type_complexity)]
pub fn button_game_over(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &GameOverButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
    mut morale: ResMut<EnemyMorale>,
    mut current_day: ResMut<CurrentDay>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
) {
    for (interaction, mut color, button_type) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                audio_player.play(audio.click.clone());
                match *button_type {
                    GameOverButton::Restart => {
                        current_day.day = 0;
                        morale.current = 50.0;
                        state.set(GameState::MoraleStatus).unwrap();
                    }
                    GameOverButton::MainMenu => {
                        state.set(GameState::MainMenu).unwrap();
                    }
                }
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

pub fn spawn_game_over(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    morale: Res<EnemyMorale>,
    current_day: Res<CurrentDay>,
) {
    let game_over_narration = if morale.current == 100.0 {
        "Recent victories have granted bravery to humanity.
\nInvigorated, they begin truly pushing you back,
giving you defeat after defeat.
\nSoon, you are driven back to your lair,
and your phylactery is destroyed,
freeing the world from your grasp."
    } else {
        "Humanity has found true despair.
\nYour crushing victories against them
have made them give up on attacking.
\nYou begin actively raiding settlements
in a desperate bid for more souls,
but soon, they will run out,
and you will perish from hunger."
    };

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
                                value: game_over_narration.to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "\n\nYour reign lasted".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: format!("\n{} days", current_day.day),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 64.0,
                                    color: Color::WHITE,
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
                .spawn_bundle(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    let spawn_button =
                        |parent: &mut ChildBuilder, text: &str, context: GameOverButton| {
                            parent
                                .spawn_bundle(ButtonBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                        margin: Rect {
                                            top: Val::Px(30.0),
                                            left: Val::Px(30.0),
                                            right: Val::Px(30.0),
                                            ..Default::default()
                                        },
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..Default::default()
                                    },
                                    color: BUTTON_NORMAL.into(),
                                    ..Default::default()
                                })
                                .insert(context)
                                .with_children(|parent| {
                                    parent.spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            text.to_string(),
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
                        };

                    spawn_button(parent, "Restart", GameOverButton::Restart);
                    spawn_button(parent, "Main Menu", GameOverButton::MainMenu);
                });
        });
}

// Credits

#[allow(clippy::type_complexity)]
pub fn button_credits_back(
    mut q_interaction: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<State<GameState>>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
) {
    for (interaction, mut color) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                audio_player.play(audio.click.clone());
                state.set(GameState::MainMenu).unwrap();
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

pub fn spawn_credits(mut commands: Commands, fonts: Res<GameFonts>, sprites: Res<GameSprites>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
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
                                value: "Credits\n".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 64.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "\nAssets:\nFont: ".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "m5x7 by Daniel Linssen\nhttps://managore.itch.io/m5x7".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "\nEnemy and Player Sprites: ".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "16x16 DungeonTileset II by 0x72\nhttps://0x72.itch.io/dungeontileset-ii\n".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "\n\nGame made by ".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "ProspectPyxis".to_string(),
                                style: TextStyle {
                                    font: fonts.main.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
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

            parent.spawn_bundle(ImageBundle {
                image: UiImage(sprites.bevy.clone()),
                transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.0)),
                style: Style {
                    margin: Rect {
                        top: Val::Px(30.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        margin: Rect {
                            top: Val::Px(30.0),
                            bottom: Val::Px(30.0),
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
                            "Back",
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

pub fn despawn_menu(
    mut commands: Commands,
    q_ui: Query<(Entity, &Ui), With<Children>>,
    q_tilemap: Query<Entity, With<Map>>,
) {
    for (ent, ui) in q_ui.iter() {
        if let Ui::Core = ui {
            commands.entity(ent).despawn_recursive();
        }
    }
    for ent in q_tilemap.iter() {
        commands.entity(ent).despawn_recursive();
    }
}
