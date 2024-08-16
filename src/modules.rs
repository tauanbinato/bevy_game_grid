use std::fmt::Display;

use avian2d::prelude::*;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::hierarchy::BuildChildren;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{default, Bundle, Commands, Component, Entity, Mesh, Rectangle, ResMut, Transform, Visibility};
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
    pub entity_connected: Option<Entity>,
    pub module_type: ModuleType,
    pub inner_grid_pos: (i32, i32),
}

#[derive(Bundle)]
pub struct ModuleBundleRigid {
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub collider_density: ColliderDensity,
    pub mass: Mass,
    pub module: Module,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Bundle)]
pub struct ModuleBundleInteractable {
    pub module: Module,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

pub fn spawn_module(
    commands: &mut Commands,
    structure_entity: Entity,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    module_type: ModuleType,
    color: Color,
    grid_pos: (i32, i32),
    translation: Vec3,
    cell_size: f32,
    mesh_scale_factor: f32,
    interactable: bool,
) {
    let module_entity: Entity;

    if !interactable {
        // Spawn the module entity
        module_entity = commands
            .spawn(ModuleBundleRigid {
                rigidbody: RigidBody::Kinematic,
                collider: Collider::rectangle(cell_size * mesh_scale_factor, cell_size * mesh_scale_factor),
                collider_density: ColliderDensity(0.0),
                mass: Mass(5000.0),
                module: Module { module_type, inner_grid_pos: grid_pos, ..default() },
                mesh_bundle: MaterialMesh2dBundle {
                    material: materials.add(ColorMaterial::from(color)),
                    mesh: meshes
                        .add(Rectangle { half_size: Vec2::splat((cell_size / 2.0) * mesh_scale_factor) })
                        .into(),
                    transform: Transform { translation, ..default() },
                    visibility: Visibility::Inherited,
                    ..default()
                },
            })
            .id();
    } else {
        // Spawn the module entity
        module_entity = commands
            .spawn(ModuleBundleInteractable {
                module: Module { module_type, inner_grid_pos: grid_pos, ..default() },
                mesh_bundle: MaterialMesh2dBundle {
                    material: materials.add(ColorMaterial::from(color)),
                    mesh: meshes
                        .add(Rectangle { half_size: Vec2::splat((cell_size / 2.0) * mesh_scale_factor) })
                        .into(),
                    transform: Transform { translation, ..default() },
                    visibility: Visibility::Inherited,
                    ..default()
                },
            })
            .id();
    }

    // Add the module as a child to the structure entity
    commands.entity(structure_entity).add_child(module_entity);
}
