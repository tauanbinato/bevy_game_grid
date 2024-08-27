// src/config.rs

use avian2d::math::Vector;
use bevy::math::Vec2;

// Global game configuration constants
pub const UNIT_SCALE: f32 = 10.0; // 10 pixels = 1 meter

// You can add more constants here as needed, for example:
pub const WINDOW_WIDTH: f32 = 1800.0;
pub const WINDOW_HEIGHT: f32 = 900.0;
pub const DEFAULT_GRAVITY: Vec2 = Vector::ZERO;
