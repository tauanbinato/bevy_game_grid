use my_game::configs::prelude::*;
use my_game::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "My Game Window".into(),
                        name: Some("bevy.app".into()),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
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
        .insert_resource(Gravity(DEFAULT_GRAVITY))
        .add_plugins((
            LoadersPlugins,
            GamePlugins { debug_enable: true },
            UtilityPlugins { debug_enable: true },
        ))
        //.add_plugins(WorldInspectorPlugin::new())
        .run();
}
