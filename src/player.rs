use crate::grid::Grid;
use crate::inputs::InputAction;
use avian2d::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::state::GameState;

const MOVE_SPEED: f32 = 250.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResource::default())
            .add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(FixedUpdate, movement_system.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct PlayerResource {
    pub grid_position: (i32, i32),
    pub is_controlling_structure: bool,
    pub inside_structure: Option<Entity>,
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid: ResMut<Grid>,
    mut player_grid_position: ResMut<PlayerResource>,
) {
    let initial_grid_position = (2, 2);
    let initial_world_position = grid.grid_to_world(initial_grid_position);

    player_grid_position.grid_position = initial_grid_position;

    let player_entity = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::circle(10.0),
            Player,
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle { radius: 10.0 }).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform: Transform {
                    translation: Vec3::new(initial_world_position.x, initial_world_position.y, 5.0),
                    ..default()
                },
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .id();

    grid.insert_new(initial_grid_position.0, initial_grid_position.1, player_entity);
}

fn movement_system(
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
