use serde::Deserialize;

use crate::mirror::MirrorOrientation;
use crate::types::Direction;

#[derive(Debug, Clone, Deserialize)]
pub struct MirrorData {
    pub row: i32,
    pub col: i32,
    pub orientation: MirrorOrientation,
    #[serde(default)]
    pub fixed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceData {
    pub row: i32,
    pub col: i32,
    pub direction: Direction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetData {
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LevelData {
    pub name: String,
    pub source: SourceData,
    pub target: TargetData,
    pub mirrors: Vec<MirrorData>,
}

use bevy::prelude::*;

/// Resource holding all available levels.
#[derive(Resource, Default)]
pub struct LevelRegistry {
    pub levels: Vec<LevelData>,
}

/// Resource tracking which level is currently being played.
#[derive(Resource)]
pub struct CurrentLevel(pub usize);

/// Loads all level JSON files from the assets/levels/ directory.
pub fn load_levels(
    mut registry: ResMut<LevelRegistry>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    let levels_dir = std::path::Path::new("assets/levels");
    let mut levels = Vec::new();

    if let Ok(entries) = std::fs::read_dir(levels_dir) {
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();

        for path in paths {
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match std::fs::read_to_string(&path) {
                    Ok(contents) => match serde_json::from_str::<LevelData>(&contents) {
                        Ok(level) => {
                            info!("Loaded level: {} from {:?}", level.name, path);
                            levels.push(level);
                        }
                        Err(e) => warn!("Failed to parse {:?}: {}", path, e),
                    },
                    Err(e) => warn!("Failed to read {:?}: {}", path, e),
                }
            }
        }
    } else {
        warn!("Could not read assets/levels/ directory");
    }

    info!("Loaded {} levels", levels.len());
    registry.levels = levels;
    next_state.set(crate::AppState::Menu);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_level() {
        let json = r#"{
            "name": "Test Level",
            "source": { "row": 0, "col": 3, "direction": "right" },
            "target": { "row": 5, "col": 7 },
            "mirrors": [
                { "row": 2, "col": 3, "orientation": "/", "fixed": false },
                { "row": 2, "col": 5, "orientation": "\\", "fixed": true },
                { "row": 5, "col": 5, "orientation": "-" }
            ]
        }"#;

        let level: LevelData = serde_json::from_str(json).unwrap();
        assert_eq!(level.name, "Test Level");
        assert_eq!(level.source.row, 0);
        assert_eq!(level.source.col, 3);
        assert_eq!(level.source.direction, Direction::Right);
        assert_eq!(level.target.row, 5);
        assert_eq!(level.target.col, 7);
        assert_eq!(level.mirrors.len(), 3);
        assert_eq!(
            level.mirrors[0].orientation,
            MirrorOrientation::ForwardSlash
        );
        assert!(!level.mirrors[0].fixed);
        assert_eq!(level.mirrors[1].orientation, MirrorOrientation::BackSlash);
        assert!(level.mirrors[1].fixed);
        assert_eq!(level.mirrors[2].orientation, MirrorOrientation::Horizontal);
        assert!(!level.mirrors[2].fixed); // default
    }
}
