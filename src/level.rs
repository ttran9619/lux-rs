use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
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

// ─── Bevy Asset: LevelManifest ──────────────────────────────

/// A Bevy asset containing all levels, loaded from a single JSON array file.
#[derive(Asset, TypePath, Debug)]
pub struct LevelManifest {
    pub levels: Vec<LevelData>,
}

/// Loader that deserializes a JSON array of levels into a LevelManifest.
#[derive(Default)]
pub struct LevelManifestLoader;

impl AssetLoader for LevelManifestLoader {
    type Asset = LevelManifest;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let levels: Vec<LevelData> = serde_json::from_slice(&bytes)?;
        Ok(LevelManifest { levels })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

// ─── Resources ──────────────────────────────────────────────

/// Resource holding all available levels.
#[derive(Resource, Default)]
pub struct LevelRegistry {
    pub levels: Vec<LevelData>,
}

/// Resource tracking which level is currently being played.
#[derive(Resource)]
pub struct CurrentLevel(pub usize);

/// Resource holding the handle to the manifest asset while it loads.
#[derive(Resource)]
pub struct ManifestHandle(pub Handle<LevelManifest>);

// ─── Systems ────────────────────────────────────────────────

/// Kicks off async loading of the level manifest.
#[cfg(target_arch = "wasm32")]
pub fn start_loading(
    mut registry: ResMut<LevelRegistry>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    let levels: Vec<LevelData> = serde_json::from_str(include_str!("../assets/levels.json"))
        .expect("embedded levels.json must be valid");
    bevy::log::info!("Loaded {} embedded levels for wasm", levels.len());
    registry.levels = levels;
    next_state.set(crate::AppState::Menu);
}

/// Kicks off async loading of the level manifest.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<LevelManifest>("levels.json");
    commands.insert_resource(ManifestHandle(handle));
}

/// Polls each frame until the manifest is loaded, then populates LevelRegistry.
#[cfg(target_arch = "wasm32")]
pub fn check_loading_complete() {}

/// Polls each frame until the manifest is loaded, then populates LevelRegistry.
#[cfg(not(target_arch = "wasm32"))]
pub fn check_loading_complete(
    mut commands: Commands,
    manifest_handle: Option<Res<ManifestHandle>>,
    mut manifests: ResMut<Assets<LevelManifest>>,
    mut registry: ResMut<LevelRegistry>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    let Some(handle_res) = manifest_handle else {
        return;
    };

    if let Some(manifest) = manifests.remove(handle_res.0.id()) {
        bevy::log::info!("Loaded {} levels from manifest", manifest.levels.len());
        registry.levels = manifest.levels;
        commands.remove_resource::<ManifestHandle>();
        next_state.set(crate::AppState::Menu);
    }
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

    #[test]
    fn test_deserialize_manifest() {
        let json = r#"[
            {
                "name": "Level One",
                "source": { "row": 0, "col": 0, "direction": "right" },
                "target": { "row": 0, "col": 7 },
                "mirrors": []
            },
            {
                "name": "Level Two",
                "source": { "row": 3, "col": 0, "direction": "down" },
                "target": { "row": 7, "col": 3 },
                "mirrors": [
                    { "row": 5, "col": 0, "orientation": "\\", "fixed": true }
                ]
            }
        ]"#;

        let levels: Vec<LevelData> = serde_json::from_str(json).unwrap();
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0].name, "Level One");
        assert_eq!(levels[1].name, "Level Two");
        assert_eq!(levels[1].mirrors.len(), 1);
    }
}
