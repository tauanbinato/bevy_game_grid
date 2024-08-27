use crate::core::prelude::GameState;
use crate::gameplay::structures_combat::despawn_entity;
use crate::prelude::{
    default, in_state, on_event, App, AssetServer, Bundle, Commands, Component, Deref, DerefMut, Entity, EventReader,
    GlobalTransform, IntoSystemConfigs, JustifyText, Plugin, Query, Res, Text, Text2dBundle, TextStyle, Time, Timer,
    TimerMode, Update, Vec3, With,
};
use crate::world::modules::{Module, ModuleTookDamageEvent};
use bevy::prelude::Transform;

pub struct DamageUiPlugin;

impl Plugin for DamageUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_damage_pop_up.run_if(on_event::<ModuleTookDamageEvent>()))
            .add_systems(Update, (damage_popup_lifetime, animate_translation).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component, Deref, DerefMut)]
struct DamagePopUp(Timer);

#[derive(Bundle)]
struct DamagePopUpBundle {
    damage_pop_up: DamagePopUp,
    text2d_bundle: Text2dBundle,
    animate_translation: AnimateTranslation,
}

#[derive(Component)]
struct AnimateTranslation;

fn spawn_damage_pop_up(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_reader: EventReader<ModuleTookDamageEvent>,
    module_query: Query<&GlobalTransform, With<Module>>,
) {
    for event in event_reader.read() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_style = TextStyle { font: font.clone(), font_size: 20.0, ..default() };
        let text_justification = JustifyText::Center;

        if let Ok(module_transform) = module_query.get(event.module_entity) {
            commands.spawn(DamagePopUpBundle {
                damage_pop_up: DamagePopUp(Timer::from_seconds(1.0, TimerMode::Once)),
                text2d_bundle: Text2dBundle {
                    text: Text::from_section(event.damage.to_string(), text_style.clone())
                        .with_justify(text_justification),
                    transform: Transform::from_translation(Vec3::new(
                        module_transform.translation().x,
                        module_transform.translation().y,
                        10.0,
                    )),
                    ..Default::default()
                },
                animate_translation: AnimateTranslation,
            });
        }
    }
}

/// This system ticks the `Timer` on the entity with the `projectile_entity`
/// component using bevy's `Time` resource to get the delta between each update.
fn damage_popup_lifetime(time: Res<Time>, mut query: Query<(Entity, &mut DamagePopUp)>, mut commands: Commands) {
    for (popup_entity, mut timer) in &mut query {
        if timer.tick(time.delta()).just_finished() {
            despawn_entity(popup_entity, &mut commands);
        }
    }
}

fn animate_translation(
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<DamagePopUp>, With<AnimateTranslation>)>,
) {
    for mut transform in &mut query {
        // Calculate offsets based on time, but increment rather than replace
        let x_offset = 1.0 * time.elapsed_seconds().sin();
        let y_offset = 1.0 * time.elapsed_seconds().cos();

        // Apply the offsets relative to the current position of the transform
        transform.translation.x += x_offset;
        transform.translation.y += y_offset;

        // Optional: you can also add a small downward drift to simulate gravity
        transform.translation.y -= 8.0 * time.delta_seconds(); // For a subtle fall effect
    }
}
