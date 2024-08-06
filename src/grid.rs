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

            .add_systems(Update, (update_grid_data, detect_grid_change).chain())
            .add_systems(PostUpdate, (debug_draw_grid_and_entities).in_set(InGameSet::Debug).chain())

            .add_systems(FixedUpdate, apply_gravity.in_set(InGameSet::EntityUpdates))
            .add_systems(FixedPostUpdate, detect_grid_change.in_set(InGameSet::EntityReads));

    }
}

#[derive(Default, Clone, Debug)]
pub struct GridProperties {
    pub gravity: f32
}

#[derive(Resource)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), GridCell>,
}

#[derive(Default, Debug, Resource)]
pub struct GridCell {
    pub entity: Option<Entity>,
    pub color: Srgba,
    pub properties: GridProperties,
}

impl Grid {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn insert(&mut self, x: i32, y: i32, entity: Entity) {
        self.cells.insert((x, y), GridCell { entity: Some(entity), color: Srgba::rgb(0.5, 0.5, 0.5), properties: GridProperties::default() });
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&GridCell> {
        self.cells.get(&(x, y))
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut GridCell> {
        self.cells.get_mut(&(x, y))
    }

    pub fn update(&mut self, x: i32, y: i32, new_entity: Entity, new_properties: GridProperties) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.entity = Some(new_entity);
            cell.properties = new_properties;
        }
    }

    pub fn clear_cell(&mut self, x: i32, y: i32) {
        self.cells.remove(&(x, y));
    }

    pub fn remove_entity_from_cell(&mut self, x: i32, y: i32) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.entity = None;
        }
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

fn setup_grid(mut commands: Commands, mut grid: ResMut<Grid>) {
    for x in 0..grid.width {
        for y in 0..grid.height {
            let properties= GridProperties { gravity: 1.0 };

            grid.cells.insert((x as i32, y as i32), GridCell {
                entity: None,
                color: Srgba::rgb(0.5, 0.5, 0.5),
                properties,
            });

        }
    }
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
    pub old_cell: (i32, i32),
    pub new_cell: (i32, i32),
}

fn detect_grid_change(
    query: Query<(&Transform), With<Player>>,
    mut grid: ResMut<Grid>,
    mut event_writer: EventWriter<PlayerGridChangeEvent>,
    mut player_grid_position: ResMut<PlayerGridPosition>,
) {

    for (transform) in &query {

        let (updated_grid_x, updated_grid_y) = grid.world_to_grid(transform.translation);
        let (old_grid_x, old_grid_y) = player_grid_position.grid_position;

        if (old_grid_x, old_grid_y) != (updated_grid_x, updated_grid_y) {
            println!("Player moved from ({}, {}) to ({}, {})", old_grid_x, old_grid_y, updated_grid_x, updated_grid_y);
            event_writer.send(PlayerGridChangeEvent {
                old_cell: (old_grid_x, old_grid_y),
                new_cell: (updated_grid_x, updated_grid_y),
            });

            // Update the player's grid position state
            player_grid_position.grid_position = (updated_grid_x, updated_grid_y);
        }
    }
}

fn update_grid_data(
    mut query: Query<(Entity, &Transform, &mut LinearVelocity), With<Player>>,
    mut grid: ResMut<Grid>,
    mut event_reader: EventReader<PlayerGridChangeEvent>,
) {
    for event in event_reader.read() {
        let PlayerGridChangeEvent { old_cell, new_cell } = *event;

        // if let Some(cell) = grid.get_mut(old_cell.0, old_cell.1) {
        //     cell.entity = None;
        // }
        //
        // if let Some(cell) = grid.get_mut(new_cell.0, new_cell.1) {
        //     cell.entity = query.single();
        // }
    }
}
fn apply_gravity(
    mut query: Query<(&Transform, &mut LinearVelocity)>,
    grid: Res<Grid>,
    time: Res<Time>,
) {
    for (transform, mut velocity) in &mut query {
        let (grid_x, grid_y) = grid.world_to_grid(transform.translation);
        if let Some(cell) = grid.get(grid_x, grid_y) {
            let gravity = cell.properties.gravity;

            if gravity == 1.0 {
                // Apply friction to simulate solid top-down movement
                let friction = 0.92;
                velocity.x *= friction;
                velocity.y *= friction;

            } else {
                // Apply gravity effect
                //velocity.y -= gravity * time.delta_seconds();

                // Apply damping to simulate inertia for cells with gravity less than 1.0
                velocity.x *= 0.98;
                velocity.y *= 0.98;
            }

            // Update the entity's position based on its velocity
            //transform.translation.x += velocity.x * time.delta_seconds();
            //transform.translation.y += velocity.y * time.delta_seconds();
        }
    }
}
