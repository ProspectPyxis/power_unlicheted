use crate::{
    common::{apply_angled_movement, check_despawn, GameSprites, MainCamera},
    enemy::{spawn_enemy, update_enemy},
    player::{player_move, player_shoot, spawn_player},
    projectile::check_projectile_collision,
};
use bevy::{core::FixedTimestep, prelude::*};
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Start,
}

pub struct GameSetup;

impl Plugin for GameSetup {
    fn build(&self, app: &mut App) {
        AssetLoader::new(GameState::AssetLoading)
            .continue_to_state(GameState::Start)
            .with_collection::<GameSprites>()
            .build(app);

        app.add_state(GameState::AssetLoading)
            .insert_resource(Msaa { samples: 1 })
            .insert_resource(WindowDescriptor {
                title: "Lich thing".to_string(),
                width: 1280.0,
                height: 960.0,
                ..Default::default()
            })
            .add_plugins(DefaultPlugins)
            .add_plugin(PhysicsPlugin::default())
            .add_system_set(
                SystemSet::on_enter(GameState::Start)
                    .with_system(setup_camera)
                    .with_system(spawn_player),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Start)
                    .with_system(apply_angled_movement)
                    .with_system(check_despawn)
                    .with_system(player_move)
                    .with_system(player_shoot)
                    .with_system(update_enemy)
                    .with_system(check_projectile_collision),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Start)
                    .with_run_criteria(FixedTimestep::step(2.0))
                    .with_system(spawn_enemy),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}
