use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use std::collections::HashMap;
use std::process::Command;
use avian2d::prelude::LinearVelocity;
use bevy::{
    core::FrameCount,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};
use crate::player::{Player, PlayerGridPosition};
use crate::schedule::InGameSet;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid::new(10, 10, 50.0))
            .init_gizmo_group::<MyGridGizmos>()
            .add_event::<PlayerGridChangeEvent>()
            .add_systems(Startup, setup_grid)

            .add_systems(Update, (detect_grid_change.in_set(InGameSet::EntityReads)).chain())
            .add_systems(PostUpdate, (debug_draw_grid_and_entities).in_set(InGameSet::Debug).chain())
            .add_systems(FixedUpdate, apply_gravity.in_set(InGameSet::EntityUpdates));

    }
}

#[derive(Clone, Debug)]
pub struct GridProperties {
    pub gravity: f32
}
impl Default for GridProperties {
    fn default() -> Self {
        Self {
            gravity: 1.0
        }
    }
}

#[derive(Resource)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), GridCell>,
}

#[derive(Debug, Resource)]
pub struct GridCell {
    pub entity: Option<Entity>,
    pub color: Srgba,
    pub properties: GridProperties,
}
impl Default for GridCell {
    fn default() -> Self {
        Self {
            entity: None,
            color: Srgba::rgb(0.5, 0.5, 0.5),
            properties: GridProperties::default(),
        }
    }
}

impl Grid {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        let mut cells = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                cells.insert((x as i32, y as i32), GridCell::default());
            }
        }
        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }

    pub fn insert_new(&mut self, x: i32, y: i32, entity: Entity) {
        self.cells.insert((x, y), GridCell { entity: Some(entity), color: Srgba::rgb(0.5, 0.5, 0.5), properties: GridProperties::default() });
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&GridCell> {
        self.cells.get(&(x, y))
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut GridCell> {
        self.cells.get_mut(&(x, y))
    }

    fn clear_cell(&mut self, x: i32, y: i32) {
        self.cells.remove(&(x, y));
    }

    fn remove_entity_from_cell(&mut self, x: i32, y: i32) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.entity = None;
        }
    }

    fn insert_entity_in_cell(&mut self, x: i32, y: i32, entity: Entity) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.entity = Some(entity);
        }
    }

    pub fn update_entity_position(&mut self, entity: Entity, new_x: i32, new_y: i32, old_X: i32, old_y: i32) {
        self.remove_entity_from_cell(old_X, old_y);
        self.insert_entity_in_cell(new_x, new_y, entity);
    }

    pub fn world_to_grid(&self, world_pos: Vec3) -> (i32, i32) {
        let half_width = self.width as f32 * self.cell_size / 2.0;
        let half_height = self.height as f32 * self.cell_size / 2.0;

        (
            ((world_pos.x + half_width) / self.cell_size).floor() as i32,
            ((half_height - world_pos.y) / self.cell_size).floor() as i32,
        )
    }

    pub fn grid_to_world(&self, grid_pos: (i32, i32)) -> Vec3 {
        let half_width = self.width as f32 * self.cell_size / 2.0;
        let half_height = self.height as f32 * self.cell_size / 2.0;

        Vec3::new(
            grid_pos.0 as f32 * self.cell_size - half_width + self.cell_size / 2.0,
            half_height - grid_pos.1 as f32 * self.cell_size - self.cell_size / 2.0,
            0.0,
        )
    }

    pub fn color_cell(&mut self, x: i32, y: i32, color: Srgba) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.color = color;
        }
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyGridGizmos {}

fn setup_grid(mut commands: Commands) {

    commands.spawn(Camera2dBundle::default());
}

fn debug_draw_grid_and_entities(
    mut gizmos: Gizmos,
    grid: Res<Grid>
) {
    // Another way to draw the grid
    gizmos.grid_2d(
        Vec2::ZERO,
        0.0,
        UVec2::new(grid.width, grid.height),
        Vec2::splat(grid.cell_size),
        Srgba::rgb(0.5, 0.5, 0.5),
    ).outer_edges();

    // let player_id = player_query.single();
    // let half_width = grid.width as f32 * grid.cell_size / 2.0;
    // let half_height = grid.height as f32 * grid.cell_size / 2.0;

    // for x in 0..grid.width as i32 {
    //     for y in 0..grid.height as i32 {
    //         let world_position = grid.grid_to_world((x, y)).truncate();
    //         let mut color = Srgba::rgb(0.5, 0.5, 0.5);
    //
    //         if let Some(cell) = grid.get(x, y) {
    //             if cell.properties.gravity != 1.0 {
    //                 color = Srgba::GREEN;
    //             }
    //
    //             if let Some(entity) = cell.entity {
    //                 if entity == player_id {
    //                     color = Srgba::RED;
    //                 } else {
    //                     color = cell.color;
    //                 }
    //             }
    //         }
    //
    //         gizmos.rect_2d(world_position, 0.0, Vec2::splat(grid.cell_size), color);
    //     }
    // }
}

#[derive(Event, Debug)]
pub struct PlayerGridChangeEvent {
    pub entity: Entity,
    pub old_cell: (i32, i32),
    pub new_cell: (i32, i32),
}

fn detect_grid_change(
    query: Query<(Entity, &Transform), With<Player>>,
    mut grid: ResMut<Grid>,
    mut event_writer: EventWriter<PlayerGridChangeEvent>,
    mut player_grid_position: ResMut<PlayerGridPosition>,
) {

    for (entity, transform) in &query {

        let (updated_grid_x, updated_grid_y) = grid.world_to_grid(transform.translation);
        let (old_grid_x, old_grid_y) = player_grid_position.grid_position;

        if (old_grid_x, old_grid_y) != (updated_grid_x, updated_grid_y) {
            println!("Player moved from ({}, {}) to ({}, {})", old_grid_x, old_grid_y, updated_grid_x, updated_grid_y);
            event_writer.send(PlayerGridChangeEvent {
                entity,
                old_cell: (old_grid_x, old_grid_y),
                new_cell: (updated_grid_x, updated_grid_y),
            });

            // Update the player's grid position state
            player_grid_position.grid_position = (updated_grid_x, updated_grid_y);

            // Update the grid with the new player position
            grid.update_entity_position(entity, updated_grid_x, updated_grid_y, old_grid_x, old_grid_y);
        }
    }
}
fn apply_gravity(
    mut query: Query<(&Transform, &mut LinearVelocity)>,
    grid: Res<Grid>,
    time: Res<Time>,
) {
    let damping_factor: f32 = 0.92; // Adjust this value to control the damping effect

    for (transform, mut velocity) in &mut query {
        let (grid_x, grid_y) = grid.world_to_grid(transform.translation);
        if let Some(cell) = grid.get(grid_x, grid_y) {
            let gravity = cell.properties.gravity;

            if gravity == 1.0 {
                // Apply damping to simulate gravity on a top-down world
                velocity.x *= damping_factor;
                velocity.y *= damping_factor;
            }
        }
    }
}
