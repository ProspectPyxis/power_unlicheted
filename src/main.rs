use bevy::prelude::*;

mod common;
mod enemy;
mod menu;
mod player;
mod projectile;
mod setup;

fn main() {
    App::new().add_plugin(setup::GameSetup).run();
}
