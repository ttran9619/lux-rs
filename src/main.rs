mod beam;
mod grid;
mod input;
mod level;
mod mirror;
mod types;
mod ui;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Playing,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Lux".to_string(),
                        resolution: bevy::window::WindowResolution::new(800.0, 700.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .init_state::<AppState>()
        .init_resource::<beam::LevelSolved>()
        .init_resource::<level::LevelRegistry>()
        .init_asset::<level::LevelManifest>()
        .init_asset_loader::<level::LevelManifestLoader>()
        // Startup: spawn camera
        .add_systems(Startup, spawn_camera)
        // Loading state: async load levels manifest
        .add_systems(OnEnter(AppState::Loading), level::start_loading)
        .add_systems(
            Update,
            level::check_loading_complete.run_if(in_state(AppState::Loading)),
        )
        // Menu state
        .add_systems(OnEnter(AppState::Menu), ui::spawn_menu)
        .add_systems(OnExit(AppState::Menu), ui::despawn_menu)
        .add_systems(
            Update,
            (
                ui::handle_menu_buttons,
                ui::update_menu_button_colors,
                ui::scroll_menu_level_list,
            )
                .run_if(in_state(AppState::Menu)),
        )
        // Playing state
        .add_systems(
            OnEnter(AppState::Playing),
            (
                grid::spawn_grid,
                grid::spawn_level_entities,
                ui::spawn_hud,
                beam::update_beam,
            )
                .chain(),
        )
        .add_systems(
            OnExit(AppState::Playing),
            (cleanup_playing, ui::despawn_hud, ui::despawn_win_overlay),
        )
        .add_systems(
            Update,
            (
                input::handle_mirror_click,
                input::handle_mirror_hover,
                beam::update_beam,
                ui::check_win_condition,
                ui::handle_back_button,
                ui::handle_reset_button,
                ui::handle_win_back_button,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Clean up all game entities when leaving the Playing state.
fn cleanup_playing(
    mut commands: Commands,
    grid_cells: Query<Entity, With<grid::GridCell>>,
    source_cells: Query<Entity, With<grid::SourceCell>>,
    target_cells: Query<Entity, With<grid::TargetCell>>,
    source_arrows: Query<Entity, With<grid::SourceArrow>>,
    mirrors: Query<Entity, With<mirror::Mirror>>,
    beam_lines: Query<Entity, With<beam::BeamLine>>,
) {
    let queries: [&dyn Fn(&mut Commands); 6] = [
        &|cmd| {
            grid_cells.iter().for_each(|e| {
                cmd.entity(e).despawn();
            })
        },
        &|cmd| {
            source_cells.iter().for_each(|e| {
                cmd.entity(e).despawn();
            })
        },
        &|cmd| {
            target_cells.iter().for_each(|e| {
                cmd.entity(e).despawn();
            })
        },
        &|cmd| {
            source_arrows.iter().for_each(|e| {
                cmd.entity(e).despawn();
            })
        },
        &|cmd| {
            mirrors.iter().for_each(|e| {
                cmd.entity(e).despawn();
            })
        },
        &|cmd| {
            beam_lines.iter().for_each(|e| {
                cmd.entity(e).despawn();
            })
        },
    ];
    for cleanup in queries {
        cleanup(&mut commands);
    }
}
