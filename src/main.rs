use bevy::prelude::*;

mod setup;

fn main() {
    App::new().add_plugin(setup::GameSetup).run();
}
