use bevy::prelude::*;

mod common;
mod enemy;
mod player;
mod setup;

fn main() {
    App::new().add_plugin(setup::GameSetup).run();
}
