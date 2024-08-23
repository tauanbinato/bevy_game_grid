use crate::grid::Grid;
use crate::state::GameState;
use crate::UNIT_SCALE;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

const MOVE_SPEED: f32 = 250.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResource::default())
            .add_systems(OnEnter(GameState::BuildingStructures), spawn_player);
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

    //player_grid_position.grid_position = initial_grid_position;

    let player_entity = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::circle(1.0 * UNIT_SCALE),
            ColliderDensity(0.0),
            Mass(100.0),
            Player,
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle { radius: 1.0 * UNIT_SCALE }).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform: Transform { translation: Vec3::new(-25.0, 0.0, 5.0), ..default() },
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .id();

    //grid.insert_new(initial_grid_position.0, initial_grid_position.1, player_entity);
}
