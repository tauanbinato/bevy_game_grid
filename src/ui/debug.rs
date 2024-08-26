use avian2d::prelude::PhysicsDebugPlugin;
use bevy::app::{App, Plugin, Startup};
use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::prelude::{default, Commands, Update};
use iyes_perf_ui::prelude::*;

#[derive(Default)]
pub struct DebugPlugin {
    pub enable: bool,
}
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PerfUiPlugin)
            // we want Bevy to measure these values for us:
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin);
        app.edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, ..default() });
        });
        if self.enable {
            app.add_systems(Startup, debug_startup).add_plugins(PhysicsDebugPlugin::default());
        }
    }
}

fn debug_startup(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot { display_labels: false, layout_horizontal: true, ..Default::default() },
        // PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}
