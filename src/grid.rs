use bevy::prelude::*;

pub const GRID_SIZE: i32 = 8;
pub const CELL_SIZE: f32 = 64.0;
pub const CELL_GAP: f32 = 2.0;
pub const CELL_TOTAL: f32 = CELL_SIZE + CELL_GAP;

/// The offset to center the grid at world origin.
fn grid_offset() -> Vec2 {
    let total = GRID_SIZE as f32 * CELL_TOTAL - CELL_GAP;
    Vec2::new(-total / 2.0, total / 2.0)
}

/// Convert grid (row, col) to world-space center position.
/// Row 0 is top, col 0 is left.
pub fn grid_to_world(row: i32, col: i32) -> Vec2 {
    let offset = grid_offset();
    Vec2::new(
        offset.x + col as f32 * CELL_TOTAL + CELL_SIZE / 2.0,
        offset.y - row as f32 * CELL_TOTAL - CELL_SIZE / 2.0,
    )
}

/// Convert world-space position to grid (row, col), or None if out of bounds.
pub fn world_to_grid(world_pos: Vec2) -> Option<(i32, i32)> {
    let offset = grid_offset();
    let local_x = world_pos.x - offset.x;
    let local_y = offset.y - world_pos.y;

    let col = (local_x / CELL_TOTAL).floor() as i32;
    let row = (local_y / CELL_TOTAL).floor() as i32;

    if (0..GRID_SIZE).contains(&row) && (0..GRID_SIZE).contains(&col) {
        // Check we're inside the cell, not in the gap
        let cell_x = local_x - col as f32 * CELL_TOTAL;
        let cell_y = local_y - row as f32 * CELL_TOTAL;
        if (0.0..=CELL_SIZE).contains(&cell_x) && (0.0..=CELL_SIZE).contains(&cell_y) {
            return Some((row, col));
        }
    }
    None
}

/// Marker component for grid cell background entities.
#[derive(Component)]
#[allow(dead_code)]
pub struct GridCell {
    pub row: i32,
    pub col: i32,
}

/// Marker for the source cell entity.
#[derive(Component)]
pub struct SourceCell;

/// Marker for the target cell entity.
#[derive(Component)]
pub struct TargetCell;

/// Marker for the source arrow indicator.
#[derive(Component)]
pub struct SourceArrow;

/// Spawns the 8x8 grid of cell backgrounds.
pub fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cell_mesh = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));
    let cell_material = materials.add(Color::srgb(0.85, 0.85, 0.85));

    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let pos = grid_to_world(row, col);
            commands.spawn((
                Mesh2d(cell_mesh.clone()),
                MeshMaterial2d(cell_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.0),
                GridCell { row, col },
            ));
        }
    }
}

use crate::level::{CurrentLevel, LevelRegistry};
use crate::mirror::{Mirror, OriginalOrientation};
use crate::types::Direction;

/// Spawns source cell, target cell, mirrors, and source arrow for the current level.
pub fn spawn_level_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    level_registry: Res<LevelRegistry>,
    current_level: Res<CurrentLevel>,
) {
    let level = &level_registry.levels[current_level.0];

    // Source cell highlight
    let source_pos = grid_to_world(level.source.row, level.source.col);
    let source_mesh = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));
    let source_material = materials.add(Color::srgb(1.0, 0.9, 0.2)); // yellow
    commands.spawn((
        Mesh2d(source_mesh),
        MeshMaterial2d(source_material),
        Transform::from_xyz(source_pos.x, source_pos.y, 1.0),
        SourceCell,
    ));

    // Source arrow
    let arrow_size = CELL_SIZE * 0.4;
    let arrow_mesh = meshes.add(Triangle2d::new(
        Vec2::new(0.0, arrow_size / 2.0),
        Vec2::new(-arrow_size / 2.0, -arrow_size / 2.0),
        Vec2::new(arrow_size / 2.0, -arrow_size / 2.0),
    ));
    let arrow_material = materials.add(Color::srgb(0.8, 0.4, 0.0)); // dark orange
    let arrow_rotation = match level.source.direction {
        Direction::Right => -std::f32::consts::FRAC_PI_2,
        Direction::Left => std::f32::consts::FRAC_PI_2,
        Direction::Up => 0.0,
        Direction::Down => std::f32::consts::PI,
    };
    commands.spawn((
        Mesh2d(arrow_mesh),
        MeshMaterial2d(arrow_material),
        Transform::from_xyz(source_pos.x, source_pos.y, 2.0)
            .with_rotation(Quat::from_rotation_z(arrow_rotation)),
        SourceArrow,
    ));

    // Target cell highlight
    let target_pos = grid_to_world(level.target.row, level.target.col);
    let target_mesh = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));
    let target_material = materials.add(Color::srgb(0.2, 0.9, 0.3)); // green
    commands.spawn((
        Mesh2d(target_mesh),
        MeshMaterial2d(target_material),
        Transform::from_xyz(target_pos.x, target_pos.y, 1.0),
        TargetCell,
    ));

    // Mirrors
    let mirror_width = CELL_SIZE * 0.8;
    let mirror_height = 4.0;

    let fixed_color = materials.add(Color::srgb(0.4, 0.4, 0.5)); // muted
    let rotatable_color = materials.add(Color::srgb(0.2, 0.5, 1.0)); // bright blue

    for mirror_data in &level.mirrors {
        let pos = grid_to_world(mirror_data.row, mirror_data.col);
        let mirror_mesh = meshes.add(Rectangle::new(mirror_width, mirror_height));
        let material = if mirror_data.fixed {
            fixed_color.clone()
        } else {
            rotatable_color.clone()
        };

        commands.spawn((
            Mesh2d(mirror_mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(pos.x, pos.y, 3.0).with_rotation(Quat::from_rotation_z(
                mirror_data.orientation.rotation_radians(),
            )),
            Mirror {
                row: mirror_data.row,
                col: mirror_data.col,
                fixed: mirror_data.fixed,
            },
            mirror_data.orientation,
            OriginalOrientation(mirror_data.orientation),
        ));
    }
}
