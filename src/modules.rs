use crate::grid::CellType;
use crate::structures::Structure;
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
    structure_component: &mut Structure,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    module_type: ModuleType,
    color: Color,
    grid_pos: (i32, i32),
    translation: Vec3,
    mesh_scale_factor: f32,
    interactable: bool,
) {
    if !interactable {
        // Spawn the module entity
        commands.entity(structure_entity).with_children(|children| {
            children.spawn(ModuleBundleRigid {
                collider: Collider::rectangle(
                    structure_component.grid.cell_size * mesh_scale_factor,
                    structure_component.grid.cell_size * mesh_scale_factor,
                ),
                collider_density: ColliderDensity(0.0),
                mass: Mass(5000.0),
                module: Module { module_type, inner_grid_pos: grid_pos, ..default() },
                mesh_bundle: MaterialMesh2dBundle {
                    material: materials.add(ColorMaterial::from(color)),
                    mesh: meshes
                        .add(Rectangle {
                            half_size: Vec2::splat((structure_component.grid.cell_size / 2.0) * mesh_scale_factor),
                        })
                        .into(),
                    transform: Transform { translation, ..default() },
                    visibility: Visibility::Inherited,
                    ..default()
                },
            });
        });
    } else {
        commands.entity(structure_entity).with_children(|children| {
            children.spawn(ModuleBundleInteractable {
                module: Module { module_type, inner_grid_pos: grid_pos, ..default() },
                mesh_bundle: MaterialMesh2dBundle {
                    material: materials.add(ColorMaterial::from(color)),
                    mesh: meshes
                        .add(Rectangle {
                            half_size: Vec2::splat((structure_component.grid.cell_size / 2.0) * mesh_scale_factor),
                        })
                        .into(),
                    transform: Transform { translation, ..default() },
                    visibility: Visibility::Inherited,
                    ..default()
                },
            });
        });
    }

    structure_component.grid.insert(grid_pos.0, grid_pos.1, CellType::Module);
}
