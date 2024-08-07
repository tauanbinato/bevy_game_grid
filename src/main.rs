use bevy::prelude::*;
use avian2d::{math::*, prelude::*};
use bevy::window::PresentMode;

mod grid;
mod player;
mod ore;
mod schedule;
mod state;
mod debug;

use grid::GridPlugin;
use player::PlayerPlugin;
use ore::OrePlugin;
use schedule::SchedulePlugin;
use state::StatePlugin;
use debug::DebugPlugin;

fn main() {
    App::new()

        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "My Game Window".into(),
                name: Some("bevy.app".into()),
                resolution: (1300., 800.).into(),
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(Gravity(Vector::ZERO))
        .add_plugins(GridPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(OrePlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(StatePlugin)
        //.add_plugins(DebugPlugin)
        .run();
}