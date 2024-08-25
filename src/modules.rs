use crate::grid::CellType;
use crate::structures::Structure;
use crate::UNIT_SCALE;
use avian2d::prelude::*;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::hierarchy::BuildChildren;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    debug, default, Bundle, Commands, Component, Entity, Event, Mesh, Rectangle, ResMut, Transform, Visibility,
};
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};

#[derive(Event)]
pub struct ModuleDestroyedEvent {
    pub destroyed_entity: Entity,
    pub inner_grid_pos: (i32, i32),
}

#[derive(Debug, Default)]
pub enum ModuleType {
    #[default]
    CommandCenter,
    Engine,
    Wall,
    Cannon,
}

#[derive(Debug)]
pub struct MaterialProperties {
    pub yield_strength: f32, // Yield Strength: The amount of stress the material can withstand before deforming.
    pub thickness: f32,      // Thickness in meters
    pub density: f32,        // Density in kg/m^2
    pub damage_threshold: f32, // Damage threshold in Newtons
}
#[derive(Debug, Default)]
pub enum ModuleMaterialType {
    #[default]
    Steel,
    Wood,
    Aluminum,
}

impl ModuleMaterialType {
    pub(crate) fn properties(&self) -> MaterialProperties {
        match self {
            ModuleMaterialType::Steel => MaterialProperties {
                yield_strength: 250000.0,  // Strength in J/m² (converted from MPa)
                thickness: 0.01,           // Thickness in meters (10 mm)
                density: 78.5,             // Surface density in kg/m² (7850 kg/m³ * 0.01 m)
                damage_threshold: 30000.0, // Approximation based on steel properties
            },
            ModuleMaterialType::Wood => MaterialProperties {
                yield_strength: 40000.0,  // Strength in J/m² (converted from MPa)
                thickness: 0.02,          // Thickness in meters (20 mm)
                density: 12.0,            // Surface density in kg/m² (600 kg/m³ * 0.02 m)
                damage_threshold: 5000.0, // Approximation for wood properties
            },
            ModuleMaterialType::Aluminum => MaterialProperties {
                yield_strength: 150000.0,  // Strength in J/m² (converted from MPa)
                thickness: 0.005,          // Thickness in meters (5 mm)
                density: 13.5,             // Surface density in kg/m² (2700 kg/m³ * 0.005 m)
                damage_threshold: 20000.0, // Approximation for aluminum properties
            },
        }
    }
}

#[derive(Debug, Default, Component)]
pub struct ModuleMaterial {
    pub structural_points: f32,
    pub material_type: ModuleMaterialType,
}

#[derive(Debug, Default, Component)]
pub struct Module {
    pub width: f32,
    pub height: f32,
    pub entity_connected: Option<Entity>,
    pub module_type: ModuleType,
    pub inner_grid_pos: (i32, i32),
}

#[derive(Bundle)]
pub struct ModuleBundleRigid {
    pub collider: Collider,
    pub collider_density: ColliderDensity,
    pub module: Module,
    pub module_material: ModuleMaterial,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
    pub external_force: ExternalForce,
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
    material_type: ModuleMaterialType,
) {
    let properties = material_type.properties();

    let unit_size = structure_component.grid.cell_size;
    let volume = (unit_size * mesh_scale_factor).powi(2) * properties.thickness; // Consider thickness in volume
    let structural_points =
        ((properties.yield_strength * volume * properties.density) / properties.damage_threshold) / UNIT_SCALE;

    if !interactable {
        // Spawn the module entity
        commands.entity(structure_entity).with_children(|children| {
            children.spawn(ModuleBundleRigid {
                collider: Collider::rectangle(
                    structure_component.grid.cell_size * mesh_scale_factor,
                    structure_component.grid.cell_size * mesh_scale_factor,
                ),
                collider_density: ColliderDensity(volume * properties.density),
                module: Module { module_type, inner_grid_pos: grid_pos, ..default() },
                module_material: ModuleMaterial { structural_points, material_type },
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
                external_force: ExternalForce::default(),
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
    structure_component.density += properties.density;
}
