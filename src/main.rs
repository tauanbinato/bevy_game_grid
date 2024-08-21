use avian2d::{math::*, prelude::*};
use bevy::app::PluginGroupBuilder;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
mod asset_loader;
mod debug;
mod grid;
mod inputs;
mod modules;
mod movement;
mod ore;
mod player;
mod schedule;
mod state;
mod structures;
mod structures_combat;
mod utils;

use asset_loader::AssetLoaderPlugin;
use debug::DebugPlugin;
use grid::GridPlugin;
use inputs::InputsPlugin;
use movement::MovementPlugin;
use ore::OrePlugin;
use player::PlayerPlugin;
use schedule::SchedulePlugin;
use state::StatePlugin;
use structures::StructuresPlugin;

/// A group of plugins that has loading assets involved
pub struct LoadersPlugins;
impl PluginGroup for LoadersPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(StatePlugin).add(SchedulePlugin).add(AssetLoaderPlugin)
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "My Game Window".into(),
                        name: Some("bevy.app".into()),
                        resolution: (1800., 900.).into(),
                        present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,my_game::player=debug,my_game::grid=debug,my_game::structure=debug".into(),
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(Gravity(Vector::ZERO))
        .add_plugins((
            LoadersPlugins,
            GridPlugin { debug_enable: true },
            InputsPlugin,
            PlayerPlugin,
            MovementPlugin,
            StructuresPlugin { debug_enable: true },
            OrePlugin,
            DebugPlugin { enable: true },
        ))
        //.add_plugins(WorldInspectorPlugin::new())
        .run();
}
