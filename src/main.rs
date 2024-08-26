use my_game::plugin_groups::{GamePlugins, LoadersPlugins, UtilityPlugins};
use my_game::prelude::*;
use my_game::*;

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
                    filter: "info,my_game::player=debug,my_game::grid=debug,my_game::structure=debug,my_game::movement=debug,my_game::modules=debug,my_game::structure_combat=debug".into(),
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::default().with_length_unit(UNIT_SCALE))
        .insert_resource(Gravity(Vector::ZERO))
        .add_plugins((
            LoadersPlugins,
            GamePlugins { debug_enable: true },
            UtilityPlugins { debug_enable: true },
        ))
        //.add_plugins(WorldInspectorPlugin::new())
        .run();
}
