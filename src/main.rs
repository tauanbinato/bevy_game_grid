use bevy::prelude::*;


mod grid;
mod player;
mod ore;

use grid::GridPlugin;
use player::PlayerPlugin;
use ore::OrePlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "My Game Window".into(),
                name: Some("bevy.app".into()),
                resolution: (1000., 800.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GridPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(OrePlugin)
        .run();
}