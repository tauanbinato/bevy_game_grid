use crate::core::prelude::*;
use crate::world::prelude::*;

use crate::configs::config::UNIT_SCALE;
use avian2d::math::Vector;
use avian2d::prelude::*;
use bevy::prelude::*;

const STRUCTURE_MOVE_SPEED: f32 = 20.0 * UNIT_SCALE; // m/s converted to pixels
const STRUCTURE_MAX_SPEED: f32 = 100.0 * UNIT_SCALE;
const PLAYER_MOVE_SPEED: f32 = 5.0 * UNIT_SCALE;
const PLAYER_DECELERATION_FACTOR: f32 = 10.0 * UNIT_SCALE;
const PLAYER_MAX_SPEED: f32 = 10.0 * UNIT_SCALE;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                player_move_system,
                structure_move_system,
                structure_rotate_system,
                player_stop_system,
                structure_stop_system,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

fn player_move_system(
    mut query: Query<&mut LinearVelocity, With<Player>>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
    player_resource: Res<PlayerResource>,
) {
    if player_resource.is_controlling_structure {
        return;
    }

    let delta_time = time.delta_seconds();

    for event in input_reader.read() {
        match event {
            InputAction::Move(direction) => {
                for mut velocity in &mut query {
                    velocity.x += direction.x * PLAYER_MOVE_SPEED * delta_time;
                    velocity.y += direction.y * PLAYER_MOVE_SPEED * delta_time;

                    // Clamp the velocity to the maximum speed
                    let new_velocity = Vec2::new(velocity.x, velocity.y).clamp_length_max(PLAYER_MAX_SPEED);
                    *velocity = LinearVelocity(new_velocity);
                }
            }
            _ => {}
        }
    }
}

fn player_stop_system(
    mut query: Query<&mut LinearVelocity, With<Player>>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let deceleration_factor = PLAYER_DECELERATION_FACTOR;

    for event in input_reader.read() {
        if matches!(event, InputAction::Break) {
            for mut velocity in &mut query {
                velocity.0 = apply_deceleration(velocity.0, deceleration_factor, delta_time);
            }
        }
    }
}

fn structure_stop_system(
    mut controlled_structure_query: Query<&mut LinearVelocity, With<ControlledByPlayer>>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let deceleration_factor = PLAYER_DECELERATION_FACTOR;

    for event in input_reader.read() {
        for (mut velocity) in &mut controlled_structure_query {
            match event {
                InputAction::Break => {
                    // Apply deceleration in the opposite direction of the current velocity
                    let mut velocity_vector = velocity.0;

                    // Check if velocity is non-zero to avoid unnecessary calculations
                    if velocity_vector.length_squared() > 0.0 {
                        // Calculate the deceleration to apply
                        let deceleration = -velocity_vector.normalize() * deceleration_factor * delta_time;

                        // Apply deceleration to the velocity
                        velocity_vector += deceleration;

                        // Prevent overshooting: Stop the player if velocity is close to zero
                        if velocity_vector.length_squared() < (deceleration_factor * delta_time).powi(2) {
                            velocity_vector = Vector::ZERO;
                        }

                        // Update the player's velocity
                        velocity.0 = velocity_vector;
                    }
                }
                _ => {}
            }
        }
    }
}

// TODO: Refactor to use observers
fn structure_move_system(
    mut controlled_structure_query: Query<
        (&mut ExternalForce, &mut LinearVelocity, &AngularVelocity, &ControlledByPlayer, &Children),
        With<Structure>,
    >,
    player_resource: ResMut<PlayerResource>,
    mut input_reader: EventReader<InputAction>,
    mut child_query: Query<&mut Module>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let mut able_to_move = false;
    if player_resource.is_controlling_structure {
        let delta_time = time.delta_seconds();

        // Get structure controlled by player should be unique
        let (mut external_force, mut structure_velocity, structure_angular_v, controlled_by, childrens) =
            controlled_structure_query.single_mut();

        for child in childrens {
            if let Ok(module) = child_query.get_mut(*child) {
                // Check if a structure has at least one engine module as child
                if matches!(module.module_type, ModuleType::Engine) {
                    able_to_move = true;
                }
            }
        }

        if able_to_move {
            for event in input_reader.read() {
                match event {
                    InputAction::Move(direction) => {
                        structure_velocity.x += direction.x * STRUCTURE_MOVE_SPEED * delta_time;
                        structure_velocity.y += direction.y * STRUCTURE_MOVE_SPEED * delta_time;

                        // Clamp the velocity to the maximum speed
                        let new_max_velocity =
                            Vec2::new(structure_velocity.x, structure_velocity.y).clamp_length_max(STRUCTURE_MAX_SPEED);
                        *structure_velocity = LinearVelocity(new_max_velocity);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn structure_rotate_system(
    mut controlled_structure_query: Query<
        (&mut AngularVelocity, &LinearVelocity),
        (With<Structure>, With<ControlledByPlayer>),
    >,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let rotation_speed = 0.1; // Base rotation speed in radians per second
    let max_rotation_speed = 0.2; // Maximum rotation speed in radians per second

    for event in input_reader.read() {
        match event {
            InputAction::Rotate(factor) => {
                if let Ok((mut structure_angular_v, structure_velocity)) = controlled_structure_query.get_single_mut() {
                    // Apply the rotation factor to the angular velocity
                    structure_angular_v.0 += factor * rotation_speed * delta_time;

                    // Clamp the angular velocity to the maximum speed
                    let new_max_angular_velocity = structure_angular_v.0.clamp(-max_rotation_speed, max_rotation_speed);
                    *structure_angular_v = AngularVelocity(new_max_angular_velocity);
                }
            }
            _ => {}
        }
    }
}

fn apply_deceleration(mut velocity: Vector, deceleration_factor: f32, delta_time: f32) -> Vector {
    if velocity.length_squared() > 0.0 {
        let deceleration = -velocity.normalize() * deceleration_factor * delta_time;
        velocity += deceleration;

        if velocity.length_squared() < (deceleration_factor * delta_time).powi(2) {
            return Vector::ZERO;
        }
    }
    velocity
}
