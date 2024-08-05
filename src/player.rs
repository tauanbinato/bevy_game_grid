use bevy::prelude::*;
use crate::grid::Grid;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle};

const MOVE_SPEED: f32 = 250.0;

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct PlayerMovedEvent {
    pub entity: Entity,
    pub new_position: Vec3,
    pub old_position: Vec3,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
        .add_systems(FixedUpdate, move_player)
        .add_event::<PlayerMovedEvent>();
    }
}

#[derive(Component)]
pub struct Player;

fn setup_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid: ResMut<Grid>,
) {
    let player_grid_position = (0, 0);
    let player_initial_position = grid.grid_to_world(player_grid_position);

    let player_entity = commands.spawn((
        Player,
        Velocity::default(),
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



fn move_player(
    mut query: Query<(Entity, &mut Transform, &mut Velocity), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut event_writer: EventWriter<PlayerMovedEvent>,
    grid: Res<Grid>,
) {
    for (entity, mut transform, mut velocity) in &mut query {
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
            let move_delta = MOVE_SPEED * direction.normalize() * time.delta_seconds();
            velocity.x += move_delta.x;
            velocity.y += move_delta.y;
        }

        // Update the player's position based on its velocity
        let old_position = transform.translation;
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();

        // Send the PlayerMovedEvent
        event_writer.send(PlayerMovedEvent {
            entity,
            old_position,
            new_position: transform.translation,
        });

        // Apply damping to simulate inertia
        velocity.x *= 0.98;
        velocity.y *= 0.98;
    }
}
