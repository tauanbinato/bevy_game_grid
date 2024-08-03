use bevy::prelude::*;
use crate::grid::Grid;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle};

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

const MOVE_SPEED: f32 = 150.0;

fn move_player(
    mut query: Query<(Entity, &mut Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut event_writer: EventWriter<PlayerMovedEvent>,
) {
    for (entity, mut transform) in &mut query {
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
            let old_position = transform.translation;
            let move_delta = MOVE_SPEED * direction.normalize() * time.delta_seconds();
            transform.translation += move_delta;

            // Send the PlayerMovedEvent
            event_writer.send(PlayerMovedEvent {
                entity,
                old_position,
                new_position: transform.translation,
            });
        }
    }
}
