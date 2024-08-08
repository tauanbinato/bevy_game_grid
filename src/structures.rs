use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::prelude::{in_state, OnEnter};
use crate::state::GameState;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct Structure {
    pub modules: HashMap<(i32, i32), Module>,
}

impl Structure {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, x: i32, y: i32, module: Module) {
        self.modules.insert((x, y), module);
    }
}

pub struct SpaceshipBuilder {
    structure: Structure,
}

impl SpaceshipBuilder {
    pub fn new() -> Self {
        Self {
            structure: Structure::new(),
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

    pub fn build(self) -> Structure {
        self.structure
    }
}

fn spawn_ship(
) {
    let spaceship = SpaceshipBuilder::new()
        .add_command_center(0, 0)
        .add_engine(1, 0)
        .add_living_quarters(0, 1)
        .add_storage(1, 1)
        .add_weapon(2, 0)
        .build();

}

pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_ship);
    }
}