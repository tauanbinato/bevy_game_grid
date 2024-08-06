use bevy::app::{App, Plugin, Startup};
use bevy::prelude::Commands;
use iyes_perf_ui::entries::PerfUiFramerateEntries;
use iyes_perf_ui::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
            app
                .add_plugins(PerfUiPlugin)
                // we want Bevy to measure these values for us:
                .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
                .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
                .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
                .add_systems(Startup, debug_startup);
    }
}

fn debug_startup(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            ..Default::default()
        },
        // PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}