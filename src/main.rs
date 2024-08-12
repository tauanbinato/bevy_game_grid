use bevy::prelude::*;
use avian2d::{math::*, prelude::*};
use bevy::app::PluginGroupBuilder;
use bevy::log::LogPlugin;
use bevy::window::PresentMode;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod grid;
mod player;
mod ore;
mod schedule;
mod state;
mod debug;
mod asset_loader;
mod structures;
mod modules;

use grid::GridPlugin;
use player::PlayerPlugin;
use ore::OrePlugin;
use schedule::SchedulePlugin;
use state::StatePlugin;
use debug::DebugPlugin;
use asset_loader::AssetLoaderPlugin;
use crate::structures::StructuresPlugin;

/// A group of plugins that has loading assets involved
pub struct LoadersPlugins;
impl PluginGroup for LoadersPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(StatePlugin)
            .add(SchedulePlugin)
            .add(AssetLoaderPlugin)
    }
}

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
            }).set(LogPlugin {
                filter: "info,my_game::player=debug,my_game::grid=debug,my_game::structure=debug".into(),
                ..default()
            })

        )
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(Gravity(Vector::ZERO))

        .add_plugins((LoadersPlugins, GridPlugin { debug_enable: true }, PlayerPlugin, StructuresPlugin { debug_enable: true }, OrePlugin, DebugPlugin { enable: true }))
        //.add_plugins(WorldInspectorPlugin::new())
        .run();
}