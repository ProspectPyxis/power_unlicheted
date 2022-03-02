use crate::{
    common::{
        check_despawn, regen_health, CurrentDay, CurrentTime, DamagePlayerEvent, DamagesEnemy,
        Enemy, EnemyMorale, GameFonts, GameSprites, GameState, InGameUI, Label, MainCamera, Player,
        Ui, WaveCore, WaveManager, SCREEN_HEIGHT, SCREEN_WIDTH,
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
use bevy::{prelude::*, render::render_resource::TextureUsages};
use bevy_asset_loader::AssetLoader;
use bevy_ecs_tilemap::prelude::*;
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
            .insert_resource(CurrentTime(Timer::from_seconds(180.0, false)))
            .add_plugins(DefaultPlugins)
            .add_plugin(PhysicsPlugin::default())
            .add_plugin(TilemapPlugin)
            .add_event::<DamagePlayerEvent>()
            .add_system(set_texture_filters_to_nearest)
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
                    .with_system(reset_timer)
                    .with_system(spawn_player)
                    .with_system(setup_ui)
                    .with_system(spawn_background),
            )
            .add_system_set(
                SystemSet::on_update(GameState::ActiveGame)
                    .with_system(player_move)
                    .with_system(player_shoot)
                    .with_system(spawn_enemy_wave)
                    .with_system(update_enemy)
                    .with_system(update_timer)
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
            )
            .add_system_set(SystemSet::on_exit(GameState::ActiveGame).with_system(despawn_all));
    }
}

fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST;
            }
        }
    }
}

fn spawn_background(mut commands: Commands, sprites: Res<GameSprites>, mut map_query: MapQuery) {
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let (mut layer_builder, _) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            MapSize(2, 2),
            ChunkSize(
                (SCREEN_WIDTH / 256.0).ceil() as u32,
                (SCREEN_HEIGHT / 256.0).ceil() as u32,
            ),
            TileSize(64.0, 64.0),
            TextureSize(64.0, 64.0),
        ),
        0u16,
        0u16,
    );

    layer_builder.set_all(TileBundle::default());
    let layer_entity = map_query.build_layer(&mut commands, layer_builder, sprites.grass.clone());

    map.add_layer(&mut commands, 0u16, layer_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(
            Transform::from_xyz(-SCREEN_WIDTH / 2.0, -SCREEN_HEIGHT / 2.0, 0.0)
                .with_scale(Vec3::new(2.0, 2.0, 0.0)),
        )
        .insert(GlobalTransform::default());
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
                sections: vec![
                    TextSection {
                        value: "Time left:\n".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/m5x7.ttf"),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "10:00\n".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/m5x7.ttf"),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Ui::TimeLeftDisplay)
        .insert(InGameUI);
}

fn reset_timer(mut current_time: ResMut<CurrentTime>) {
    current_time.0.reset();
}

fn update_timer(
    time: Res<Time>,
    mut current_time: ResMut<CurrentTime>,
    mut state: ResMut<State<GameState>>,
) {
    current_time.0.tick(time.delta());
    if current_time.0.finished() {
        state.set(GameState::MoraleStatus).unwrap();
    }
}

fn update_ui(mut q_text_ui: Query<(&Ui, &mut Text)>, current_time: Res<CurrentTime>) {
    for (ui, mut text) in q_text_ui.iter_mut() {
        if let Ui::TimeLeftDisplay = ui {
            let time_remaining = current_time.time_remaining().as_secs_f32().ceil() as u32;
            text.sections[1].value = format!("{}:{:02}", time_remaining / 60, time_remaining % 60);
        }
    }
}

#[allow(clippy::type_complexity)]
fn despawn_all(
    mut commands: Commands,
    q_enemies: Query<
        Entity,
        Or<(
            With<WaveCore>,
            With<Enemy>,
            With<Player>,
            With<Map>,
            With<InGameUI>,
            With<DamagesEnemy>,
        )>,
    >,
) {
    for ent in q_enemies.iter() {
        commands.entity(ent).despawn_recursive();
    }
}
