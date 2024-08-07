use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use crate::grid::{Grid, GridPlugin};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle};
use crate::schedule::InGameSet;
use crate::state::GameState;

const MOVE_SPEED: f32 = 250.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerGridPosition::default())
            .add_event::<InputAction>()
            .add_systems(Startup, spawn_player.run_if(in_state(GameState::InGame)))
            .add_systems(Update, keyboard_input.run_if(in_state(GameState::InGame)))
            .add_systems(FixedUpdate, movement_system.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct PlayerGridPosition {
    pub grid_position: (i32, i32),
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid: ResMut<Grid>,
    mut player_grid_position: ResMut<PlayerGridPosition>
) {
    let initial_grid_position = (0, 0);
    let initial_world_position = grid.grid_to_world(initial_grid_position);

    player_grid_position.grid_position = initial_grid_position;

    let player_entity = commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(10.0),
        Player,
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle { radius: 10.0}).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform {
                translation: Vec3::new(initial_world_position.x, initial_world_position.y, 1.0),
                ..default()
            },
            ..default()
        },
    ))
        .id();

    grid.insert_new(initial_grid_position.0, initial_grid_position.1, player_entity);
}


/// An event sent for a movement input action.
#[derive(Event)]
pub enum InputAction {
    Move(Vec3),
}

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<InputAction>,
    keys: Res<ButtonInput<KeyCode>>,
) {
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
        movement_event_writer.send(InputAction::Move(direction.normalize()));
    }
}
fn movement_system(
    mut query: Query<(&mut LinearVelocity), With<Player>>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds();


    for event in input_reader.read() {

        for (mut velocity) in &mut query {

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
