use crate::inputs::InputAction;
use crate::modules::{Module, ModuleType};
use crate::player::{Player, PlayerResource};
use crate::state::GameState;
use crate::structures::{ControlledByPlayer, Structure};
use avian2d::math::Vector;
use avian2d::prelude::*;
use bevy::prelude::*;

const MOVE_SPEED: f32 = 250.0;
const DECELERATION_FACTOR: f32 = 25.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (player_move_system, structure_move_system, player_stop_system, structure_stop_system)
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
        for mut velocity in &mut query {
            match event {
                InputAction::Move(direction) => {
                    velocity.x += direction.x * MOVE_SPEED * delta_time;
                    velocity.y += direction.y * MOVE_SPEED * delta_time;
                }
                _ => {}
            }
        }
    }
}

fn player_stop_system(
    mut query: Query<&mut LinearVelocity, With<Player>>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let deceleration_factor = DECELERATION_FACTOR;

    for event in input_reader.read() {
        if matches!(event, InputAction::Break) {
            for mut velocity in &mut query {
                velocity.0 = apply_deceleration(velocity.0, deceleration_factor, delta_time);
            }
        }
    }
}

fn structure_stop_system(
    mut controlled_structure_query: Query<&mut LinearVelocity, With<crate::structures::ControlledByPlayer>>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let deceleration_factor = DECELERATION_FACTOR;

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
        (&mut LinearVelocity, &AngularVelocity, &ControlledByPlayer, &Children),
        With<Structure>,
    >,
    mut player_query: Query<(&mut LinearVelocity, &mut AngularVelocity), (With<Player>, Without<Structure>)>,
    player_resource: ResMut<PlayerResource>,
    mut input_reader: EventReader<InputAction>,
    mut child_query: Query<&mut Module>,
    time: Res<Time>,
) {
    let mut able_to_move = false;
    if player_resource.is_controlling_structure {
        let delta_time = time.delta_seconds();
        // Get structure controlled by player should be unique
        let (mut structure_velocity, structure_angular_v, controlled_by, childrens) =
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
            if let Ok((mut player_velocity, mut player_angular_vel)) = player_query.get_mut(controlled_by.player_entity)
            {
                for event in input_reader.read() {
                    match event {
                        InputAction::Move(direction) => {
                            structure_velocity.x += direction.x * 100.0 * delta_time;
                            structure_velocity.y += direction.y * 100.0 * delta_time;
                        }
                        _ => {}
                    }
                }
                *player_velocity = structure_velocity.clone();
                *player_angular_vel = structure_angular_v.clone();
            }
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
