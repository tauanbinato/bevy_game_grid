use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use crate::grid::Grid;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle};
use crate::schedule::InGameSet;

const MOVE_SPEED: f32 = 250.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMoveEvent>().add_event::<InputAction>()
            .add_systems(PostStartup, spawn_player.in_set(InGameSet::SpawnEntities))
            .add_systems(Update, keyboard_input.in_set(InGameSet::UserInput))
            .add_systems(FixedUpdate, movement_system.in_set(InGameSet::EntityUpdates));
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid: ResMut<Grid>,
) {
    let player_grid_position = (0, 0);
    let player_initial_position = grid.grid_to_world(player_grid_position);

    let player_entity = commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(10.0),
        Player,
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle { radius: 10.0}).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform {
                translation: Vec3::new(player_initial_position.x, player_initial_position.y, 1.0),
                ..default()
            },
            ..default()
        },
    ))
        .id();

    grid.insert(player_grid_position.0, player_grid_position.1, player_entity);
}


/// An event sent for a movement input action.
#[derive(Event)]
pub enum InputAction {
    Move(Vec3),
}

#[derive(Event)]
pub struct PlayerMoveEvent {
    pub entity: Entity,
    pub old_position: Vec3,
    pub new_position: Vec3,
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
    mut query: Query<(Entity, &Transform, &mut LinearVelocity), With<Player>>,
    mut movement_event_reader: EventReader<InputAction>,
    mut movement_event_writer: EventWriter<PlayerMoveEvent>,
    time: Res<Time>,
    grid: Res<Grid>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for event in movement_event_reader.read() {

        for (entity, mut transform, mut velocity) in &mut query {

            match event {
                InputAction::Move(direction) => {
                    velocity.x += direction.x * MOVE_SPEED * delta_time;
                    velocity.y += direction.y * MOVE_SPEED * delta_time;

                    // Update the player's position based on its velocity
                    let old_position = transform.translation;
                    let new_position = Vec3::new(
                        old_position.x + velocity.x * delta_time,
                        old_position.y + velocity.y * delta_time,
                        old_position.z,
                    );

                    // Send the PlayerMovedEvent
                    movement_event_writer.send(PlayerMoveEvent {
                        entity,
                        old_position,
                        new_position,
                    });
                }
                _ => {}
            }



        }
    }


}
