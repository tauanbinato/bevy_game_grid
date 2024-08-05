use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use crate::grid::Grid;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle};

pub struct OrePlugin;

impl Plugin for OrePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ore);
    }
}

#[derive(Component)]
pub struct Ore;

fn setup_ore(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid: ResMut<Grid>,
) {
    let ore_grid_position = (5, 5);
    let ore_initial_position = grid.grid_to_world(ore_grid_position);

    let ore_entity = commands.spawn((
        RigidBody::Static,
        Collider::circle(10.0),
        Ore,
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle { radius: 10.0}).into(),
            material: materials.add(ColorMaterial::from(Color::srgba(0.0, 1.0, 0.0, 1.0))),
            transform: Transform {
                translation: Vec3::new(ore_initial_position.x, ore_initial_position.y, 0.0),
                ..default()
            },
            ..default()
        },
    ))
        .id();

    grid.insert(ore_grid_position.0, ore_grid_position.1, ore_entity);
}