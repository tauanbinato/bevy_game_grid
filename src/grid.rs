use crate::asset_loader::{AssetBlob, AssetStore, Level};
use crate::player::{Player, PlayerGridPosition};
use crate::state::GameState;
use avian2d::collision::Collider;
use avian2d::prelude::{LinearVelocity, RigidBody};
use bevy::prelude::*;
use bevy::{color::palettes::css::*, sprite::MaterialMesh2dBundle};
use std::collections::HashMap;

#[derive(Default)]
pub struct GridPlugin {
    pub debug_enable: bool,
}

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<MyGridGizmos>()
            .add_event::<PlayerGridChangeEvent>()
            .add_systems(OnEnter(GameState::BuildingGrid), setup_grid_from_file)
            .add_systems(FixedUpdate, apply_gravity.run_if(in_state(GameState::InGame)))
            .add_systems(Update, detect_grid_updates.run_if(in_state(GameState::InGame)));

        if self.debug_enable {
            app.add_systems(
                Update,
                (detect_grid_updates, debug_draw_grid, debug_draw_rects).chain().run_if(in_state(GameState::InGame)),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentType {
    OuterSpace { gravity: f32 },
    InsideShip { gravity: f32 },
    PlanetSurface { gravity: f32 },
    Wall { gravity: f32 },
}

impl From<char> for EnvironmentType {
    fn from(c: char) -> Self {
        match c {
            'S' => EnvironmentType::InsideShip { gravity: 1.0 },
            'P' => EnvironmentType::PlanetSurface { gravity: 1.0 },
            '#' => EnvironmentType::OuterSpace { gravity: 0.0 },
            'W' => EnvironmentType::Wall { gravity: 1.0 },
            _ => EnvironmentType::OuterSpace { gravity: 0.0 },
        }
    }
}

#[derive(Debug)]
pub struct GridProperties {
    pub environment: EnvironmentType,
}
impl Default for GridProperties {
    fn default() -> Self {
        Self { environment: EnvironmentType::InsideShip { gravity: 1.0 } }
    }
}

#[derive(Resource, Default, Debug)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), GridCell>,
}

#[derive(Debug, Resource)]
pub struct GridCell {
    pub data: Option<Entity>,
    pub color: Srgba,
    pub properties: GridProperties,
}
impl Default for GridCell {
    fn default() -> Self {
        Self { data: None, color: Srgba::rgb(0.5, 0.5, 0.5), properties: GridProperties::default() }
    }
}

impl Grid {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        let mut cells: HashMap<(i32, i32), GridCell> = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                cells.insert((x as i32, y as i32), GridCell::default());
            }
        }
        Self { width, height, cell_size, cells }
    }

    pub fn insert_new(&mut self, x: i32, y: i32, data: Entity) {
        self.cells.insert(
            (x, y),
            GridCell { data: Some(data), color: Srgba::rgb(0.5, 0.5, 0.5), properties: GridProperties::default() },
        );
    }

    pub fn insert(&mut self, x: i32, y: i32) {
        self.cells.insert(
            (x, y),
            GridCell { data: None, color: Srgba::rgb(0.5, 0.5, 0.5), properties: GridProperties::default() },
        );
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&GridCell> {
        self.cells.get(&(x, y))
    }

    fn remove_entity_from_cell(&mut self, x: i32, y: i32) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.data = None;
        }
    }

    fn insert_entity_in_cell(&mut self, x: i32, y: i32, data: Entity) {
        if let Some(cell) = self.cells.get_mut(&(x, y)) {
            cell.data = Some(data);
        }
    }

    pub fn update_data_position(&mut self, data: Entity, new_x: i32, new_y: i32, old_x: i32, old_y: i32) {
        self.remove_entity_from_cell(old_x, old_y);
        self.insert_entity_in_cell(new_x, new_y, data);
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
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyGridGizmos {}

fn setup_grid_from_file(
    mut commands: Commands,
    asset_store: Res<AssetStore>,
    blob_assets: Res<Assets<AssetBlob>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    if let Some(blob) = blob_assets.get(&asset_store.level_blob) {
        let level_data: String = String::from_utf8(blob.bytes.clone()).expect("Invalid UTF-8 data");
        let level: Level = serde_json::from_str(&level_data).expect("Failed to deserialize level data");

        let mut cells = HashMap::new();
        debug!("Loading level with width: {}, height: {}, cell_size: {}", level.width, level.height, level.cell_size);
        for (y, row) in level.world.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                let environment = EnvironmentType::from(cell);

                let cell_world_pos = Vec3::new(
                    (x as f32 * level.cell_size) - (level.width as f32 * level.cell_size) / 2.0 + level.cell_size / 2.0,
                    (level.height as f32 * level.cell_size) / 2.0
                        - (y as f32 * level.cell_size)
                        - level.cell_size / 2.0,
                    0.0,
                );

                if let EnvironmentType::Wall { .. } = environment {
                    commands.spawn((
                        RigidBody::Static,
                        Collider::rectangle(level.cell_size, level.cell_size),
                        MaterialMesh2dBundle {
                            mesh: meshes.add(Rectangle { half_size: Vec2::splat(level.cell_size / 2.0) }).into(),
                            material: materials.add(Color::from(GREY)),
                            transform: Transform {
                                translation: Vec3::new(cell_world_pos.x, cell_world_pos.y, 0.0),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                }

                cells.insert(
                    (x as i32, y as i32),
                    GridCell {
                        data: None,
                        color: Srgba::rgb(0.5, 0.5, 0.5),
                        properties: GridProperties { environment },
                    },
                );
            }
        }
        let grid: Grid = Grid { width: level.width, height: level.height, cell_size: level.cell_size, cells };
        commands.insert_resource(grid);
        next_state.set(GameState::BuildingStructures);
    } else {
        panic!("Failed to load level asset");
    }
}

fn debug_draw_grid(mut gizmos: Gizmos, grid: Res<Grid>) {
    // Another way to draw the grid
    gizmos
        .grid_2d(
            Vec2::ZERO,
            0.0,
            UVec2::new(grid.width, grid.height),
            Vec2::splat(grid.cell_size),
            Srgba::rgb(0.5, 0.5, 0.5),
        )
        .outer_edges();
}

fn debug_draw_rects(mut gizmos: Gizmos, grid: Res<Grid>, query: Query<&Transform, With<Player>>) {
    let square_size = grid.cell_size * 0.95; // Adjust this value to control the size of the square

    for transform in &query {
        let (grid_x, grid_y) = grid.world_to_grid(transform.translation);

        // Draw a red rectangle at the player's current grid position
        let world_pos = grid.grid_to_world((grid_x, grid_y));
        gizmos.rect_2d(Vec2::new(world_pos.x, world_pos.y), 0.0, Vec2::splat(square_size), Srgba::RED);
    }
}

#[derive(Event, Debug)]
pub struct PlayerGridChangeEvent {
    pub entity: Entity,
    pub old_cell: (i32, i32),
    pub new_cell: (i32, i32),
}

fn detect_grid_updates(
    query: Query<(Entity, &Transform), With<Player>>,
    mut grid: ResMut<Grid>,
    mut event_writer: EventWriter<PlayerGridChangeEvent>,
    mut player_grid_position: ResMut<PlayerGridPosition>,
) {
    for (entity, transform) in &query {
        let (updated_grid_x, updated_grid_y) = grid.world_to_grid(transform.translation);
        let (old_grid_x, old_grid_y) = player_grid_position.grid_position;

        if (old_grid_x, old_grid_y) != (updated_grid_x, updated_grid_y) {
            // debug!("Player moved from ({}, {}) to ({}, {})", old_grid_x, old_grid_y, updated_grid_x, updated_grid_y);
            event_writer.send(PlayerGridChangeEvent {
                entity,
                old_cell: (old_grid_x, old_grid_y),
                new_cell: (updated_grid_x, updated_grid_y),
            });

            // Update the player's grid position state
            player_grid_position.grid_position = (updated_grid_x, updated_grid_y);

            // Update the grid with the new player position
            grid.update_data_position(entity, updated_grid_x, updated_grid_y, old_grid_x, old_grid_y);
        }
    }
}
fn apply_gravity(mut query: Query<(&Transform, &mut LinearVelocity)>, grid: Res<Grid>) {
    let damping_factor: f32 = 0.95; // Adjust this value to control the damping effect

    for (transform, mut velocity) in &mut query {
        let (grid_x, grid_y) = grid.world_to_grid(transform.translation);
        if let Some(cell) = grid.get(grid_x, grid_y) {
            let gravity = match cell.properties.environment {
                EnvironmentType::OuterSpace { gravity } => gravity,
                EnvironmentType::InsideShip { gravity } => gravity,
                EnvironmentType::PlanetSurface { gravity } => gravity,
                EnvironmentType::Wall { gravity } => gravity,
            };

            if gravity == 1.0 {
                // Apply damping to simulate gravity on a top-down world
                velocity.x *= damping_factor;
                velocity.y *= damping_factor;
            }
        }
    }
}
