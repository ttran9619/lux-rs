use bevy::prelude::*;
use serde::Deserialize;

use crate::types::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Component)]
pub enum MirrorOrientation {
    #[serde(rename = "/")]
    ForwardSlash,
    #[serde(rename = "\\")]
    BackSlash,
    #[serde(rename = "-")]
    Horizontal,
    #[serde(rename = "|")]
    Vertical,
}

impl MirrorOrientation {
    /// Cycle: / → | → \ → - → /
    pub fn rotate(self) -> Self {
        match self {
            Self::ForwardSlash => Self::Vertical,
            Self::Vertical => Self::BackSlash,
            Self::BackSlash => Self::Horizontal,
            Self::Horizontal => Self::ForwardSlash,
        }
    }

    /// Returns the new beam direction after hitting this mirror,
    /// or None if the beam passes through.
    pub fn reflect(self, incoming: Direction) -> Option<Direction> {
        match (self, incoming) {
            // Forward slash /
            (Self::ForwardSlash, Direction::Right) => Some(Direction::Up),
            (Self::ForwardSlash, Direction::Left) => Some(Direction::Down),
            (Self::ForwardSlash, Direction::Down) => Some(Direction::Left),
            (Self::ForwardSlash, Direction::Up) => Some(Direction::Right),

            // Back slash \
            (Self::BackSlash, Direction::Right) => Some(Direction::Down),
            (Self::BackSlash, Direction::Left) => Some(Direction::Up),
            (Self::BackSlash, Direction::Down) => Some(Direction::Right),
            (Self::BackSlash, Direction::Up) => Some(Direction::Left),

            // Horizontal -
            (Self::Horizontal, Direction::Down) => Some(Direction::Up),
            (Self::Horizontal, Direction::Up) => Some(Direction::Down),
            (Self::Horizontal, Direction::Right) => None, // pass through
            (Self::Horizontal, Direction::Left) => None,  // pass through

            // Vertical |
            (Self::Vertical, Direction::Right) => Some(Direction::Left),
            (Self::Vertical, Direction::Left) => Some(Direction::Right),
            (Self::Vertical, Direction::Down) => None, // pass through
            (Self::Vertical, Direction::Up) => None,   // pass through
        }
    }

    /// Rotation angle in radians for rendering the mirror sprite.
    pub fn rotation_radians(self) -> f32 {
        match self {
            Self::ForwardSlash => std::f32::consts::FRAC_PI_4, // 45°
            Self::BackSlash => -std::f32::consts::FRAC_PI_4,   // -45°
            Self::Horizontal => 0.0,
            Self::Vertical => std::f32::consts::FRAC_PI_2, // 90°
        }
    }
}

/// Marker component for a mirror entity on the grid.
#[derive(Component)]
pub struct Mirror {
    pub row: i32,
    pub col: i32,
    pub fixed: bool,
}

/// Marker for the original orientation (for reset).
#[derive(Component)]
pub struct OriginalOrientation(pub MirrorOrientation);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_cycle() {
        let start = MirrorOrientation::ForwardSlash;
        let r1 = start.rotate();
        assert_eq!(r1, MirrorOrientation::Vertical);
        let r2 = r1.rotate();
        assert_eq!(r2, MirrorOrientation::BackSlash);
        let r3 = r2.rotate();
        assert_eq!(r3, MirrorOrientation::Horizontal);
        let r4 = r3.rotate();
        assert_eq!(r4, MirrorOrientation::ForwardSlash);
    }

    #[test]
    fn test_forward_slash_reflections() {
        let m = MirrorOrientation::ForwardSlash;
        assert_eq!(m.reflect(Direction::Right), Some(Direction::Up));
        assert_eq!(m.reflect(Direction::Left), Some(Direction::Down));
        assert_eq!(m.reflect(Direction::Down), Some(Direction::Left));
        assert_eq!(m.reflect(Direction::Up), Some(Direction::Right));
    }

    #[test]
    fn test_back_slash_reflections() {
        let m = MirrorOrientation::BackSlash;
        assert_eq!(m.reflect(Direction::Right), Some(Direction::Down));
        assert_eq!(m.reflect(Direction::Left), Some(Direction::Up));
        assert_eq!(m.reflect(Direction::Down), Some(Direction::Right));
        assert_eq!(m.reflect(Direction::Up), Some(Direction::Left));
    }

    #[test]
    fn test_horizontal_pass_through() {
        let m = MirrorOrientation::Horizontal;
        assert_eq!(m.reflect(Direction::Right), None);
        assert_eq!(m.reflect(Direction::Left), None);
        assert_eq!(m.reflect(Direction::Down), Some(Direction::Up));
        assert_eq!(m.reflect(Direction::Up), Some(Direction::Down));
    }

    #[test]
    fn test_vertical_pass_through() {
        let m = MirrorOrientation::Vertical;
        assert_eq!(m.reflect(Direction::Right), Some(Direction::Left));
        assert_eq!(m.reflect(Direction::Left), Some(Direction::Right));
        assert_eq!(m.reflect(Direction::Down), None);
        assert_eq!(m.reflect(Direction::Up), None);
    }
}
