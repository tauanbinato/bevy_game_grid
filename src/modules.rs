use avian2d::prelude::*;
use bevy::prelude::{Bundle, Component, Entity};
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};

#[derive(Debug, Default)]
pub enum ModuleType {
    #[default]
    CommandCenter,
    Engine,
    Wall,
}

#[derive(Debug, Default, Component)]
pub struct Module {
    pub module_type: ModuleType,
    pub inner_grid_pos: (i32, i32),
}

#[derive(Bundle)]
pub struct ModuleBundle {
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub module: Module,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}
