use crate::{
    common::{
        check_despawn, regen_health, CurrentDay, DamagePlayerEvent, EnemyMorale, GameFonts,
        GameSprites, GameState, Label, MainCamera, Ui, WaveManager, SCREEN_HEIGHT, SCREEN_WIDTH,
    },
    enemy::{
        check_enemy_player_collision, despawn_enemies, enemy_damage_player, spawn_enemy_wave,
        update_enemy, update_enemy_render,
    },
    menu::{
        button_shift_narration, button_start_day, despawn_menu, spawn_menu, spawn_morale_status,
    },
    player::{
        player_move, player_shoot, register_player_damage, spawn_player, update_health_display,
    },
    projectile::check_projectile_collision,
};
use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;

pub struct GameSetup;

impl Plugin for GameSetup {
    fn build(&self, app: &mut App) {
        AssetLoader::new(GameState::AssetLoading)
            .continue_to_state(GameState::Opening)
            .with_collection::<GameSprites>()
            .with_collection::<GameFonts>()
            .build(app);

        app.add_state(GameState::AssetLoading)
            .insert_resource(Msaa { samples: 1 })
            .insert_resource(WindowDescriptor {
                title: "Power Unlicheted".to_string(),
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                resizable: false,
                ..Default::default()
            })
            .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
            .insert_resource(EnemyMorale(50.0))
            .insert_resource(WaveManager {
                active_waves: 0,
                max_waves: 4,
                wave_timer: Timer::from_seconds(3.0, false),
            })
            .insert_resource(CurrentDay(0))
            .add_plugins(DefaultPlugins)
            .add_plugin(PhysicsPlugin::default())
            .add_event::<DamagePlayerEvent>()
            .add_system_set(
                SystemSet::on_enter(GameState::Opening)
                    .with_system(setup_camera)
                    .with_system(spawn_menu),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Opening).with_system(button_shift_narration),
            )
            .add_system_set(SystemSet::on_exit(GameState::Opening).with_system(despawn_menu))
            .add_system_set(
                SystemSet::on_enter(GameState::MoraleStatus).with_system(spawn_morale_status),
            )
            .add_system_set(
                SystemSet::on_update(GameState::MoraleStatus).with_system(button_start_day),
            )
            .add_system_set(SystemSet::on_exit(GameState::MoraleStatus).with_system(despawn_menu))
            .add_system_set(
                SystemSet::on_enter(GameState::ActiveGame)
                    .with_system(spawn_player)
                    .with_system(setup_ui),
            )
            .add_system_set(
                SystemSet::on_update(GameState::ActiveGame)
                    .with_system(player_move)
                    .with_system(player_shoot)
                    .with_system(spawn_enemy_wave)
                    .with_system(update_enemy)
                    .label(Label::Movement),
            )
            .add_system_set(
                SystemSet::on_update(GameState::ActiveGame)
                    .with_system(check_enemy_player_collision)
                    .with_system(check_projectile_collision)
                    .label(Label::CollisionCheck)
                    .after(Label::Movement),
            )
            .add_system_set(
                SystemSet::on_update(GameState::ActiveGame)
                    .with_system(enemy_damage_player)
                    .with_system(register_player_damage)
                    .with_system(regen_health)
                    .label(Label::HealthUpdate)
                    .after(Label::CollisionCheck),
            )
            .add_system_set(
                SystemSet::on_update(GameState::ActiveGame)
                    .with_system(check_despawn)
                    .with_system(despawn_enemies)
                    .label(Label::Despawn)
                    .after(Label::HealthUpdate),
            )
            .add_system_set(
                SystemSet::on_update(GameState::ActiveGame)
                    .with_system(update_health_display)
                    .with_system(update_enemy_render)
                    .with_system(update_ui)
                    .label(Label::UpdateSprites)
                    .after(Label::Despawn),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "50.0%".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/m5x7.ttf"),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Ui::MoraleDisplay);
}

fn update_ui(mut q_text_ui: Query<(&Ui, &mut Text)>, morale: Res<EnemyMorale>) {
    for (ui, mut text) in q_text_ui.iter_mut() {
        if let Ui::MoraleDisplay = ui {
            text.sections[0].value = format!("{:.1}%", morale.0);
        }
    }
}
