use crate::asset_loader::{AssetBlob, AssetStore, StructuresData};
use crate::grid::{CellType, Grid};
use crate::inputs::InputAction;
use crate::modules::{spawn_module, Module, ModuleType};
use crate::player::{Player, PlayerResource};
use crate::state::GameState;
use avian2d::prelude::*;
use bevy::app::{App, Plugin, Update};
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use bevy::prelude::*;
use log::debug;
use std::collections::{HashSet, VecDeque};

#[derive(Default)]
pub struct StructuresPlugin {
    pub debug_enable: bool,
}

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ModuleInteractionEvent>()
            .add_event::<StructureInteractionEvent>()
            .add_systems(OnEnter(GameState::BuildingStructures), setup_structures_from_file)
            .add_systems(
                Update,
                (control_command_center_system, update_pressurization_system).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PostUpdate,
                (detect_player_inside_structure_system, make_player_child_of_structure_system)
                    .chain()
                    .after(PhysicsSet::Sync)
                    .run_if(in_state(GameState::InGame)),
            );

        if self.debug_enable {
            app.add_systems(
                PostUpdate,
                (debug_draw_structure_grid, debug_draw_player_inside_structure_rect)
                    .after(PhysicsSet::Sync)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
        }
    }
}

#[derive(Component)]
pub(crate) struct ControlledByPlayer {
    pub(crate) player_entity: Entity,
}

#[derive(Component)]
struct StructureSensor(Entity);

#[derive(Bundle)]
struct StructureBundle {
    rigid_body: RigidBody,
    collider: Collider,
    collision_margin: CollisionMargin,
    structure: Structure,
    spatial_bundle: SpatialBundle,
    collision_layers: CollisionLayers,
}

#[derive(Component, Debug)]
pub struct Structure {
    pub grid: Grid,
}

impl Structure {
    pub fn new() -> Self {
        Structure { grid: Default::default() }
    }

    /// Converts a world position into the grid coordinates of the structure.
    fn world_to_grid(&self, world_pos: Vec3, structure_transform: &Transform) -> (i32, i32) {
        let local_pos = Structure::world_to_local_grid_position(world_pos.truncate(), structure_transform);

        let grid_x =
            ((local_pos.x + (self.grid.width as f32 * self.grid.cell_size) / 2.0) / self.grid.cell_size).floor() as i32;

        // Notice here that we negate the local Y position to flip the Y axis
        let grid_y = (((self.grid.height as f32 * self.grid.cell_size) / 2.0 - local_pos.y) / self.grid.cell_size)
            .floor() as i32;

        (grid_x, grid_y)
    }

    /// Converts a world position into the local grid space of the structure.
    fn world_to_local_grid_position(world_pos: Vec2, structure_transform: &Transform) -> Vec2 {
        let structure_world_pos = structure_transform.translation.truncate();
        let z_rotation = structure_transform.rotation.to_euler(EulerRot::XYZ).2;

        // Translate and rotate the world position to local grid space
        let translated_player_pos = world_pos - structure_world_pos;
        let rotation_matrix = Mat2::from_angle(-z_rotation); // Inverse rotation
        rotation_matrix * translated_player_pos
    }

    /// Given grid cell coordinates, returns the world position of the center of that cell.
    fn grid_cell_center_world_position(&self, cell_x: i32, cell_y: i32, structure_transform: &Transform) -> Vec2 {
        let structure_world_pos = structure_transform.translation.truncate();
        let z_rotation = structure_transform.rotation.to_euler(EulerRot::XYZ).2;

        // Calculate the local position of the cell center, taking the flipped y-axis into account
        let cell_local_pos = Vec2::new(
            (cell_x as f32 - self.grid.width as f32 / 2.0) * self.grid.cell_size + self.grid.cell_size / 2.0,
            -((cell_y as f32 - self.grid.height as f32 / 2.0) * self.grid.cell_size + self.grid.cell_size / 2.0),
        );

        // Apply rotation to the cell's local position
        let rotated_cell_pos = Mat2::from_angle(z_rotation) * cell_local_pos;

        // Calculate the final world position of the cell
        structure_world_pos + rotated_cell_pos
    }

    /// Checks if the given grid coordinates are within the bounds of the structure's grid.
    pub fn is_within_grid_bounds(&self, grid_x: i32, grid_y: i32) -> bool {
        grid_x >= 0 && grid_x < self.grid.width as i32 && grid_y >= 0 && grid_y < self.grid.height as i32
    }

    pub fn check_pressurization(&self) -> HashSet<(i32, i32)> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start flood fill from all cells on the boundary that are not modules
        for x in 0..self.grid.width as i32 {
            for y in &[0, self.grid.height as i32 - 1] {
                if let Some(cell) = self.grid.get(x, *y) {
                    if cell.cell_type != CellType::Module {
                        queue.push_back((x, *y));
                    }
                }
            }
        }

        for y in 0..self.grid.height as i32 {
            for x in &[0, self.grid.width as i32 - 1] {
                if let Some(cell) = self.grid.get(*x, y) {
                    if cell.cell_type != CellType::Module {
                        queue.push_back((*x, y));
                    }
                }
            }
        }

        // Perform flood fill
        while let Some((x, y)) = queue.pop_front() {
            if visited.contains(&(x, y)) {
                continue;
            }

            visited.insert((x, y));

            for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx;
                let ny = y + dy;

                if self.is_within_grid_bounds(nx, ny) {
                    if let Some(cell) = self.grid.get(nx, ny) {
                        if cell.cell_type != CellType::Module && !visited.contains(&(nx, ny)) {
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }
        }

        visited
    }
}

fn setup_structures_from_file(
    mut commands: Commands,
    asset_store: Res<AssetStore>,
    blob_assets: Res<Assets<AssetBlob>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(blob) = blob_assets.get(&asset_store.structures_blob) {
        let structures_data: String = String::from_utf8(blob.bytes.clone()).expect("Invalid UTF-8 data");
        let structures: StructuresData =
            serde_json::from_str(&structures_data).expect("Failed to deserialize structures data");

        for structure_data in structures.structures {
            let mut structure_component = Structure::new();

            let grid_width = structure_data[0].len() as f32;
            let grid_height = structure_data.len() as f32;

            debug!("Grid width: {}, Grid height: {}", grid_width, grid_height);

            let mesh_scale_factor = 0.90; // Adjust this value to reduce the mesh size

            structure_component.grid = Grid::new(
                grid_width as u32,  // Width of the structure
                grid_height as u32, // Height of the structure
                50.0,               // Cell size
            );

            let structure_entity = commands.spawn_empty().id();
            let structure_transform = Transform::from_translation(Vec3::new(0.0, 200.0, 1.0));

            for (y, row) in structure_data.iter().enumerate() {
                for (x, cell) in row.chars().enumerate() {
                    let x_translation = ((x as f32 - (grid_width / 2.0)) * structure_component.grid.cell_size)
                        + (structure_component.grid.cell_size / 2.0);
                    let y_translation = ((grid_height / 2.0) - y as f32) * structure_component.grid.cell_size
                        - (structure_component.grid.cell_size / 2.0);

                    // Match the character to determine the type of module to spawn
                    match cell {
                        'E' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut structure_component,
                                &mut materials,
                                &mut meshes,
                                ModuleType::Engine,
                                Color::from(RED),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, 1.0),
                                mesh_scale_factor,
                                false,
                            );
                        }
                        'W' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut structure_component,
                                &mut materials,
                                &mut meshes,
                                ModuleType::Wall,
                                Color::from(GREY),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, 1.0),
                                mesh_scale_factor,
                                false,
                            );
                        }
                        'C' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut structure_component,
                                &mut materials,
                                &mut meshes,
                                ModuleType::CommandCenter,
                                Color::from(BLUE),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, -1.0),
                                mesh_scale_factor,
                                true,
                            );
                        }
                        _ => continue, // Skip characters that don't correspond to a module
                    };
                }
            }

            commands.entity(structure_entity).with_children(|children| {
                children.spawn((
                    StructureSensor(structure_entity),
                    Collider::rectangle(
                        grid_width * structure_component.grid.cell_size,
                        grid_height * structure_component.grid.cell_size,
                    ),
                    Transform { translation: Vec3::new(0.0, 0.0, 2.0), ..default() },
                    Sensor,
                ));
            });

            // Insert the structure bundle
            commands.entity(structure_entity).insert(StructureBundle {
                rigid_body: RigidBody::Dynamic,
                collision_layers: CollisionLayers::NONE,
                collider: Collider::rectangle(
                    grid_width * structure_component.grid.cell_size,
                    grid_height * structure_component.grid.cell_size,
                ),
                collision_margin: CollisionMargin(0.1),
                structure: structure_component,
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(structure_transform.translation),
                    visibility: Visibility::Visible,
                    ..Default::default()
                },
            });
        }
        next_state.set(GameState::InGame);
    } else {
        panic!("Failed to load structures asset");
    }
}

#[derive(Event)]
pub enum ModuleInteractionEvent {
    TakeControl { player_entity: Entity, structure_entity: Entity },
    ReleaseControl { player_entity: Entity, structure_entity: Entity },
}

#[derive(Event)]
pub enum StructureInteractionEvent {
    PlayerEntered { player_entity: Entity, structure_entity: Entity },
    PlayerExited { player_entity: Entity, structure_entity: Entity },
}

// TODO: USE OBSERVER INSTEAD OF SYSTEM
fn make_player_child_of_structure_system(
    mut event_reader: EventReader<StructureInteractionEvent>,
    mut command: Commands,
) {
    for event in event_reader.read() {
        match event {
            StructureInteractionEvent::PlayerEntered { player_entity, structure_entity } => {
                command.entity(*player_entity).set_parent_in_place(*structure_entity);
                debug!("Player is now a child of the structure.");
            }
            StructureInteractionEvent::PlayerExited { player_entity, structure_entity: _ } => {
                command.entity(*player_entity).remove_parent_in_place();
                debug!("Player is no longer a child of the structure.");
            }
        }
    }
}

fn update_pressurization_system(mut gizmos: Gizmos, query: Query<(&Transform, &Structure)>) {
    for (structure_transform, structure) in query.iter() {
        let grid = &structure.grid;
        let exposed_cells = structure.check_pressurization();

        // Iterate over all cells in the grid
        for y in 0..grid.height as i32 {
            for x in 0..grid.width as i32 {
                // Get the cell and check its type
                if let Some(cell) = grid.get(x, y) {
                    // Skip drawing if the cell is a Wall or a Module
                    if matches!(cell.cell_type, CellType::Module) {
                        continue;
                    }

                    let is_pressurized = !exposed_cells.contains(&(x, y));

                    // Determine the cell color based on pressurization status
                    let color = if is_pressurized {
                        Color::rgb(0.0, 1.0, 0.0) // Green for pressurized
                    } else {
                        Color::rgb(1.0, 0.0, 0.0) // Red for unpressurized
                    };

                    // Calculate the world position of the cell's center
                    let cell_world_pos = structure.grid_cell_center_world_position(x, y, structure_transform);

                    // Draw the rectangle for the cell
                    gizmos.rect_2d(
                        cell_world_pos,
                        structure_transform.rotation.to_euler(EulerRot::XYZ).2,
                        Vec2::splat(grid.cell_size * 0.70), // Slightly smaller to avoid overlapping
                        color,
                    );
                }
            }
        }
    }
}

fn control_command_center_system(
    mut event_reader: EventReader<InputAction>,
    mut event_writer: EventWriter<ModuleInteractionEvent>,
    mut player_query: Query<(Entity, &GlobalTransform, &mut LinearVelocity), With<Player>>,
    mut command: Commands,
    mut parent_query: Query<(Entity, &Structure, &Transform, &Children)>,
    mut child_query: Query<&mut Module>,
    mut player_resource: ResMut<PlayerResource>,
) {
    //loop for player pos
    for (player_entity, player_transform, mut player_velocity) in &mut player_query {
        for (structure_entity, structure, structure_transform, children) in &mut parent_query {
            // Convert the adjusted position to grid coordinates
            let (player_grid_x, player_grid_y) =
                structure.world_to_grid(player_transform.translation(), structure_transform);

            // Check if the player's grid coordinates are within the grid's bounds
            if structure.is_within_grid_bounds(player_grid_x, player_grid_y) {
                // Player is inside the structure's grid at this point.
                // Check if the player is in a Command Center and if so, check if the player is already controlling it
                for child in children {
                    if let Ok(mut module) = child_query.get_mut(*child) {
                        if matches!(module.module_type, ModuleType::CommandCenter)
                            && matches!((module.inner_grid_pos.0, module.inner_grid_pos.1), (x, y) if x == player_grid_x && y == player_grid_y)
                        {
                            // Player can control or release the Command Center by pressing the spacebar.
                            for event in event_reader.read() {
                                if let InputAction::SpacePressed = event {
                                    if module.entity_connected.is_none() {
                                        *player_velocity = LinearVelocity::ZERO; // Stop the player's movement

                                        // Take control if no one is controlling it
                                        module.entity_connected = Some(player_entity);
                                        debug!("Player is now controlling the Command Center.");

                                        // let's insert the PlayerControlled component to the structure
                                        command.entity(structure_entity).insert(ControlledByPlayer { player_entity });

                                        // Update the player resource to indicate that the player is controlling a structure
                                        player_resource.is_controlling_structure = true;

                                        // Emit an event for taking control
                                        event_writer.send(ModuleInteractionEvent::TakeControl {
                                            player_entity,
                                            structure_entity,
                                        });
                                    } else if module.entity_connected == Some(player_entity) {
                                        // Release control if the player is already controlling it
                                        module.entity_connected = None;
                                        debug!("Player has released control of the Command Center.");

                                        // let's remove the PlayerControlled component from the structure
                                        command.entity(structure_entity).remove::<ControlledByPlayer>();

                                        // Update the player resource to indicate that the player is not controlling a structure
                                        player_resource.is_controlling_structure = false;

                                        // Emit an event for releasing control
                                        event_writer.send(ModuleInteractionEvent::ReleaseControl {
                                            player_entity,
                                            structure_entity,
                                        });
                                    }
                                }
                            }
                        } else {
                            //debug!("Player is not in a Command Center or the is not the module.");
                        }
                    }
                }
            }
        }
    }
}

fn detect_player_inside_structure_system(
    player_query: Query<(Entity, &GlobalTransform, &Player)>,
    structures_query: Query<(Entity, &Transform, &Structure)>,
    mut event_writer: EventWriter<StructureInteractionEvent>,
    mut player_resource: ResMut<PlayerResource>,
) {
    for (player_entity, player_transform, _player) in &player_query {
        for (structure_entity, structure_transform, structure) in &structures_query {
            // Convert player's world position to the structure's grid coordinates
            let (player_grid_x, player_grid_y) =
                structure.world_to_grid(player_transform.translation(), structure_transform);

            // Check if the player's grid coordinates are within the grid's bounds
            if structure.is_within_grid_bounds(player_grid_x, player_grid_y) {
                // Emit an event to indicate that the player is inside the structure only if the player is not already inside
                if player_resource.inside_structure != Some(structure_entity) {
                    player_resource.inside_structure = Some(structure_entity);
                    event_writer.send(StructureInteractionEvent::PlayerEntered { player_entity, structure_entity });
                }
            } else {
                // Emit an event to indicate that the player has exited the structure only if the player was inside
                if player_resource.inside_structure == Some(structure_entity) {
                    player_resource.inside_structure = None;
                    event_writer.send(StructureInteractionEvent::PlayerExited { player_entity, structure_entity });
                }
            }
        }
    }
}

fn debug_draw_structure_grid(mut gizmos: Gizmos, structures_query: Query<(&Transform, &Structure)>) {
    for (structure_transform, structure) in &structures_query {
        // Iterate through each cell in the grid
        for y in 0..structure.grid.height {
            for x in 0..structure.grid.width {
                // Get the world position of the cell's center
                let cell_world_pos = structure.grid_cell_center_world_position(x as i32, y as i32, structure_transform);

                // Draw the rectangle for the cell
                gizmos.rect_2d(
                    cell_world_pos,
                    structure_transform.rotation.to_euler(EulerRot::XYZ).2,
                    Vec2::splat(structure.grid.cell_size * 0.95),
                    Color::from(GREY),
                );
            }
        }
    }
}

fn debug_draw_player_inside_structure_rect(
    mut gizmos: Gizmos,
    player_query: Query<(&GlobalTransform, &Player)>,
    structures_query: Query<(&Transform, &Structure)>,
) {
    for (player_transform, _player) in &player_query {
        for (structure_transform, structure) in &structures_query {
            // Convert player's world position to the structure's grid coordinates
            let (player_grid_x, player_grid_y) =
                structure.world_to_grid(player_transform.translation(), structure_transform);

            // Check if the player's grid coordinates are within the grid's bounds
            if structure.is_within_grid_bounds(player_grid_x, player_grid_y) {
                // Get the world position of the cell's center where the player is located
                let cell_world_pos =
                    structure.grid_cell_center_world_position(player_grid_x, player_grid_y, structure_transform);

                // Draw the green rectangle inside the cell where the player is located
                gizmos.rect_2d(
                    cell_world_pos,
                    structure_transform.rotation.to_euler(EulerRot::XYZ).2,
                    Vec2::splat(structure.grid.cell_size * 0.95),
                    Color::srgb(0.0, 1.0, 0.0), // Green color
                );
            }
        }
    }
}
