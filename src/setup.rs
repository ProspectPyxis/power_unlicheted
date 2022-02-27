use crate::{
    player::{player_move, spawn_player},
    GameAssets,
};
use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;

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
            .with_collection::<GameAssets>()
            .build(app);

        app.add_state(GameState::AssetLoading)
            .insert_resource(WindowDescriptor {
                title: "Lich thing".to_string(),
                width: 1280.0,
                height: 960.0,
                ..Default::default()
            })
            .add_plugins(DefaultPlugins)
            .add_system_set(
                SystemSet::on_enter(GameState::Start)
                    .with_system(setup_camera)
                    .with_system(spawn_player),
            )
            .add_system_set(SystemSet::on_update(GameState::Start).with_system(player_move));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
