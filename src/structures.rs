use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Assets, Circle, ColorMaterial, Commands, Component, default, Gizmos, in_state, Mesh, OnEnter, Query, ResMut, Transform, Vec2};
use crate::state::GameState;
use avian2d::prelude::RigidBody;
use bevy::color::Color;
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use bevy::sprite::MaterialMesh2dBundle;
use crate::grid::Grid;

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
    pub health: f32,
    pub power_consumption: f32,
}

impl Module {
    pub fn new(module_type: ModuleType, health: f32, power_consumption: f32) -> Self {
        Self {
            module_type,
            health,
            power_consumption,
        }
    }
}


#[derive(Component)]
pub struct Structure {
    pub grid: Grid<Module>,
    pub universe_pos: Transform,
    pub universe_rotation: f32,
}

impl Structure {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        Self {
            grid: Grid::new(width, height, cell_size),
            universe_pos: Transform::from_translation(Vec3::ZERO),
            universe_rotation: 0.0,
        }
    }

    pub fn add_module(&mut self, x: i32, y: i32, module: Module) {
        self.grid.insert_new(x, y, module);
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.universe_pos = Transform::from_translation(Vec3::new(position.x, position.y, 0.0));
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.universe_rotation = rotation;
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
        self.structure.add_module(x, y, Module::new(ModuleType::Engine, 100.0, 20.0));
        self
    }

    pub fn add_command_center(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::CommandCenter, 100.0, 10.0));
        self
    }

    pub fn add_living_quarters(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::LivingQuarters, 100.0, 5.0));
        self
    }

    pub fn add_storage(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::Storage, 100.0, 5.0));
        self
    }

    pub fn add_weapon(mut self, x: i32, y: i32) -> Self {
        self.structure.add_module(x, y, Module::new(ModuleType::Weapon, 100.0, 15.0));
        self
    }

    pub fn set_position(mut self, position: Vec2) -> Self {
        self.structure.set_position(position);
        self
    }

    pub fn set_rotation(mut self, rotation: f32) -> Self {
        self.structure.set_rotation(rotation);
        self
    }

    pub fn build(self) -> Structure {
        self.structure
    }
}


pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_ship)
            .add_systems(Update, debug_draw_structure_grid);
    }
}

fn spawn_ship(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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