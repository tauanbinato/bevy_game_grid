use bevy::prelude::*;

use crate::core::state::GameState;

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputAction>().add_systems(Update, keyboard_input.run_if(in_state(GameState::InGame)));
    }
}

/// An event sent for a player input action.
#[derive(Event)]
pub enum InputAction {
    Break,
    Move(Vec3),
    SpacePressed,
    Shoot,
    Rotate(f32), // Rotation factor: positive for clockwise, negative for counterclockwise
}

fn keyboard_input(mut input_event_writer: EventWriter<InputAction>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_released(KeyCode::Space) {
        input_event_writer.send(InputAction::SpacePressed);
    }

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if direction.length() > 0.0 {
        input_event_writer.send(InputAction::Move(direction.normalize()));
    }

    if keys.pressed(KeyCode::KeyX) {
        input_event_writer.send(InputAction::Break);
    }

    if keys.just_pressed(KeyCode::KeyG) {
        input_event_writer.send(InputAction::Shoot);
    }

    // Handle rotation with rotation factor
    if keys.pressed(KeyCode::KeyQ) {
        input_event_writer.send(InputAction::Rotate(1.0)); // Counterclockwise rotation
    }
    if keys.pressed(KeyCode::KeyE) {
        input_event_writer.send(InputAction::Rotate(-1.0)); // Clockwise rotation
    }
}
