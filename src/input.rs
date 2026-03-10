use bevy::prelude::*;

use crate::grid;
use crate::mirror::{Mirror, MirrorOrientation};

/// System that handles mouse clicks on mirrors to rotate them.
pub fn handle_mirror_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut mirror_query: Query<(&Mirror, &mut MirrorOrientation, &mut Transform)>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_pos) = get_world_cursor_pos(&windows, &camera_query) else {
        return;
    };

    let Some((row, col)) = grid::world_to_grid(cursor_pos) else {
        return;
    };

    for (mirror, mut orientation, mut transform) in mirror_query.iter_mut() {
        if mirror.row == row && mirror.col == col && !mirror.fixed {
            let new_orientation = orientation.rotate();
            *orientation = new_orientation;
            transform.rotation = Quat::from_rotation_z(new_orientation.rotation_radians());
        }
    }
}

/// Hover highlight system — changes color of hovered rotatable mirrors.
pub fn handle_mirror_hover(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut materials_query: Query<(&Mirror, &mut MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cursor_pos = get_world_cursor_pos(&windows, &camera_query);
    let hovered_cell = cursor_pos.and_then(grid::world_to_grid);

    let rotatable_color = Color::srgb(0.2, 0.5, 1.0);
    let hover_color = Color::srgb(0.5, 0.75, 1.0);
    let fixed_color = Color::srgb(0.4, 0.4, 0.5);

    for (mirror, mat_handle) in materials_query.iter_mut() {
        let is_hovered = hovered_cell
            .map(|(r, c)| r == mirror.row && c == mirror.col)
            .unwrap_or(false);

        let color = if mirror.fixed {
            fixed_color
        } else if is_hovered {
            hover_color
        } else {
            rotatable_color
        };

        // Update the material color
        if let Some(mat) = materials.get_mut(mat_handle.0.id()) {
            mat.color = color;
        }
    }
}

fn get_world_cursor_pos(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    camera
        .viewport_to_world_2d(camera_transform, cursor_pos)
        .ok()
}
