use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Assets, ColorMaterial, Commands, Component, Gizmos, in_state, IntoSystemConfigs, Mesh, OnEnter, Query, ResMut, Transform, Vec2, With};
use crate::state::GameState;
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use crate::grid::Grid;
use crate::player::Player;

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    Engine,
    CommandCenter,
    LivingQuarters,
    Storage,
    Weapon,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub module_type: ModuleType,
}

impl Module {
    pub fn new(module_type: ModuleType) -> Self {
        Self {
            module_type,
        }
    }
}


#[derive(Component)]
pub struct Structure {
    pub grid: Grid<Module>,
    pub universe_pos: Transform,
}

impl Structure {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        Self {
            grid: Grid::new(width, height, cell_size),
            universe_pos: Transform::from_translation(Vec3::ZERO),
        }
    }

    pub fn add_module(&mut self, x: i32, y: i32, module: Module) {
        self.grid.insert_new(x, y, module);
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.universe_pos = Transform::from_translation(Vec3::new(position.x, position.y, 0.0));
    }
}

pub struct SpaceshipBuilder {
    structure: Structure,
}

impl SpaceshipBuilder {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        Self {
            structure: Structure::new(width, height, cell_size),
        }
    }

    pub fn add_engine(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::Engine));
        self
    }

    pub fn add_command_center(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::CommandCenter));
        self
    }

    pub fn add_living_quarters(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::LivingQuarters));
        self
    }

    pub fn add_storage(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::Storage));
        self
    }

    pub fn add_weapon(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::Weapon));
        self
    }

    pub fn set_position(mut self, position: Vec2) -> Self {
        self.structure.set_position(position);
        self
    }

    pub fn build(self) -> Structure {
        self.structure
    }
}

#[derive(Default)]
pub struct StructuresPlugin {
    pub(crate) debug_enable: bool,
}

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_ship);

        if self.debug_enable {
            app.add_systems(Update, (debug_draw_structure_grid,
                                     debug_draw_player_in_structure).chain().run_if(in_state(GameState::InGame)));
        }
    }
}

fn spawn_ship(
    mut commands: Commands,
) {
    let mut spaceship = SpaceshipBuilder::new(3, 3, 50.0)
        .add_command_center(0, 0)
        .add_engine(1, 0)
        .add_living_quarters(0, 1)
        .add_storage(1, 1)
        .add_weapon(2, 0)
        .build();

    spaceship.set_position(Vec2::new(500.0, 50.0));

    let spaceship = commands.spawn(spaceship).id();
}

fn debug_draw_structure_grid(
    mut gizmos: Gizmos,
    query: Query<&Structure>,
) {
    for structure in &query {
        let grid = &structure.grid;
        let universe_pos = structure.universe_pos.translation;

        for ((x, y), cell) in &grid.cells {

            let mut world_pos = grid.grid_to_world((*x, *y));
            world_pos += universe_pos;
            let mut color = GREY;
            if let Some(module) = &cell.data {
                color  = match module.module_type {
                    ModuleType::Engine => RED,
                    ModuleType::CommandCenter => BLUE,
                    ModuleType::LivingQuarters => GREEN,
                    ModuleType::Storage => YELLOW,
                    ModuleType::Weapon => PURPLE,
                };
            }

            gizmos.rect_2d(
                Vec2::new(world_pos.x, world_pos.y),
                0.0,
                Vec2::splat(grid.cell_size * 0.95),
                color,
            );
        }
    }
}


// New system to debug draw player position within the structure's grid
fn debug_draw_player_in_structure(
    mut gizmos: Gizmos,
    structure_query: Query<&Structure>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_color = GREEN;


    for structure in &structure_query {
        let grid = &structure.grid;
        let universe_pos = structure.universe_pos.translation;

        // Draw player position within the structure's grid
        for player_transform in &player_query {
            let player_world_pos = player_transform.translation - universe_pos;
            let player_grid_pos = grid.world_to_grid(player_world_pos);
            let square_size = grid.cell_size * 0.90; // Adjust this value to control the size of the square

            // Check if the player is within the grid boundaries
            if player_grid_pos.0 >= 0 && player_grid_pos.0 < grid.width as i32 &&
                player_grid_pos.1 >= 0 && player_grid_pos.1 < grid.height as i32 {
                let player_world_pos = grid.grid_to_world(player_grid_pos) + universe_pos;
                gizmos.rect_2d(
                    Vec2::new(player_world_pos.x, player_world_pos.y),
                    0.0,
                    Vec2::splat(square_size),
                    player_color,
                );
            }
        }
    }
}