use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

mod player;
mod setup;

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(path = "sprites", folder)]
    _sprites: Vec<HandleUntyped>,
}

fn main() {
    App::new().add_plugin(setup::GameSetup).run();
}
