use bevy::prelude::*;
use std::collections::HashSet;

use crate::grid::{self, GRID_SIZE};
use crate::mirror::{Mirror, MirrorOrientation};
use crate::types::{Direction, GridPos};

/// A segment of the beam between two world-space points.
#[derive(Debug, Clone)]
pub struct BeamSegment {
    pub start: Vec2,
    pub end: Vec2,
}

/// Result of beam tracing.
pub struct BeamTraceResult {
    pub segments: Vec<BeamSegment>,
    pub reached_target: bool,
}

/// Traces the beam through the grid, returning segments and whether it hit the target.
pub fn trace_beam(
    source_row: i32,
    source_col: i32,
    source_dir: Direction,
    target_row: i32,
    target_col: i32,
    mirrors: &[(i32, i32, MirrorOrientation)],
) -> BeamTraceResult {
    let mut segments = Vec::new();
    let mut pos = GridPos::new(source_row, source_col);
    let mut dir = source_dir;
    let mut visited: HashSet<(i32, i32, Direction)> = HashSet::new();
    let mut reached_target = false;

    let mut segment_start = grid::grid_to_world(pos.row, pos.col);

    const MAX_STEPS: usize = 200;
    for _ in 0..MAX_STEPS {
        // Check for cycle
        if !visited.insert((pos.row, pos.col, dir)) {
            break;
        }

        // Check if we reached the target
        if pos.row == target_row && pos.col == target_col {
            let world = grid::grid_to_world(pos.row, pos.col);
            segments.push(BeamSegment {
                start: segment_start,
                end: world,
            });
            reached_target = true;
            break;
        }

        // Check for mirror at current position
        if let Some(orientation) = mirrors
            .iter()
            .find(|(r, c, _)| *r == pos.row && *c == pos.col)
            .map(|(_, _, o)| *o)
            && let Some(new_dir) = orientation.reflect(dir)
        {
            // Reflection: end current segment, start a new one
            let world = grid::grid_to_world(pos.row, pos.col);
            segments.push(BeamSegment {
                start: segment_start,
                end: world,
            });
            segment_start = world;
            dir = new_dir;
        }

        // Step to next cell
        let next = pos.step(dir);
        if !next.in_bounds(GRID_SIZE) {
            // Beam exits grid — end segment at grid edge
            let world = grid::grid_to_world(pos.row, pos.col);
            segments.push(BeamSegment {
                start: segment_start,
                end: world,
            });
            break;
        }
        pos = next;
    }

    BeamTraceResult {
        segments,
        reached_target,
    }
}

/// Marker component for beam line entities.
#[derive(Component)]
pub struct BeamLine;

/// Resource indicating whether the current level is solved.
#[derive(Resource, Default, PartialEq)]
pub struct LevelSolved(pub bool);

/// System that recomputes and renders the beam whenever mirrors change.
pub fn update_beam(
    mut commands: Commands,
    beam_query: Query<Entity, With<BeamLine>>,
    mirror_query: Query<(&Mirror, &MirrorOrientation)>,
    level_registry: Res<crate::level::LevelRegistry>,
    current_level: Res<crate::level::CurrentLevel>,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut level_solved: ResMut<LevelSolved>,
) {
    // Despawn old beam lines
    for entity in beam_query.iter() {
        commands.entity(entity).despawn();
    }

    let level = &level_registry.levels[current_level.0];

    // Collect current mirror states
    let mirrors: Vec<(i32, i32, MirrorOrientation)> = mirror_query
        .iter()
        .map(|(m, &o)| (m.row, m.col, o))
        .collect();

    let result = trace_beam(
        level.source.row,
        level.source.col,
        level.source.direction,
        level.target.row,
        level.target.col,
        &mirrors,
    );

    if level_solved.0 != result.reached_target {
        level_solved.0 = result.reached_target;
    }

    let beam_color = if result.reached_target {
        Color::srgb(0.2, 1.0, 0.3) // green when solved
    } else {
        Color::srgb(1.0, 0.7, 0.0) // orange normally
    };
    let beam_material = materials.add(beam_color);

    for segment in &result.segments {
        let mid = (segment.start + segment.end) / 2.0;
        let delta = segment.end - segment.start;
        let length = delta.length();
        if length < 0.1 {
            continue;
        }
        let angle = delta.y.atan2(delta.x);
        let beam_thickness = 3.0;

        let mesh = meshes.add(Rectangle::new(length, beam_thickness));
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(beam_material.clone()),
            Transform::from_xyz(mid.x, mid.y, 5.0).with_rotation(Quat::from_rotation_z(angle)),
            BeamLine,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_beam_no_mirrors() {
        let result = trace_beam(0, 0, Direction::Right, 0, 7, &[]);
        assert!(result.reached_target);
    }

    #[test]
    fn test_beam_with_reflection() {
        // Beam goes right, hits a / mirror at (0,3), should go up — exits grid
        let mirrors = vec![(0, 3, MirrorOrientation::ForwardSlash)];
        let result = trace_beam(0, 0, Direction::Right, 0, 7, &mirrors);
        assert!(!result.reached_target);
    }

    #[test]
    fn test_beam_reaches_target_via_mirrors() {
        // Beam goes right from (0,0), hits \ at (0,2) → goes down,
        // hits / at (3,2) → goes left, target at (3,0)
        let mirrors = vec![
            (0, 2, MirrorOrientation::BackSlash),
            (3, 2, MirrorOrientation::ForwardSlash),
        ];
        let result = trace_beam(0, 0, Direction::Right, 3, 0, &mirrors);
        assert!(result.reached_target);
    }

    #[test]
    fn test_beam_pass_through() {
        // Beam goes right, horizontal mirror at (0,2) — should pass through
        let mirrors = vec![(0, 2, MirrorOrientation::Horizontal)];
        let result = trace_beam(0, 0, Direction::Right, 0, 7, &mirrors);
        assert!(result.reached_target);
    }

    #[test]
    fn test_beam_exits_grid() {
        let result = trace_beam(0, 0, Direction::Up, 7, 7, &[]);
        assert!(!result.reached_target);
    }
}
