use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::AppState;
use crate::beam::LevelSolved;
use crate::level::{CurrentLevel, LevelRegistry};

// ─── Menu UI ────────────────────────────────────────────────

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct LevelButton(pub usize);

#[derive(Component)]
pub struct MenuLevelList;

pub fn spawn_menu(mut commands: Commands, level_registry: Res<LevelRegistry>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            MenuRoot,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Lux"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Scroll to see all levels"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.82, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Scrollable level list to avoid clipping on short screens.
            parent
                .spawn((
                    Node {
                        width: Val::Px(340.0),
                        max_height: Val::Percent(65.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.14, 0.14, 0.2)),
                    BorderColor(Color::srgb(0.35, 0.35, 0.45)),
                    ScrollPosition::default(),
                    MenuLevelList,
                ))
                .with_children(|list| {
                    for (i, level) in level_registry.levels.iter().enumerate() {
                        list.spawn((
                            Button,
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
                            LevelButton(i),
                        ))
                        .with_child((
                            Text::new(format!("{}. {}", i + 1, level.name)),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    }
                });

            parent.spawn((
                Text::new("Use mouse wheel or touchpad"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.74, 0.82)),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
}

pub fn scroll_menu_level_list(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut level_list_query: Query<&mut ScrollPosition, With<MenuLevelList>>,
) {
    let Ok(mut scroll_pos) = level_list_query.single_mut() else {
        return;
    };

    let mut delta_y = 0.0;
    for event in mouse_wheel_events.read() {
        delta_y += match event.unit {
            MouseScrollUnit::Line => event.y * 32.0,
            MouseScrollUnit::Pixel => event.y,
        };
    }

    if delta_y != 0.0 {
        scroll_pos.offset_y = (scroll_pos.offset_y - delta_y).max(0.0);
    }
}

pub fn handle_menu_buttons(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &LevelButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, level_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(CurrentLevel(level_button.0));
            next_state.set(AppState::Playing);
        }
    }
}

type MenuButtonColorQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (With<LevelButton>, Changed<Interaction>),
>;

pub fn update_menu_button_colors(mut query: MenuButtonColorQuery) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.35, 0.35, 0.5)),
            Interaction::None => BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
        };
    }
}

pub fn despawn_menu(mut commands: Commands, menu_query: Query<Entity, With<MenuRoot>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

// ─── In-Game HUD ────────────────────────────────────────────

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct BackToMenuButton;

#[derive(Component)]
pub struct ResetButton;

pub fn spawn_hud(
    mut commands: Commands,
    level_registry: Res<LevelRegistry>,
    current_level: Res<CurrentLevel>,
) {
    let level = &level_registry.levels[current_level.0];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
            HudRoot,
        ))
        .with_children(|parent| {
            // Top bar with level name and buttons
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(60.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(16.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|top_bar| {
                    // Back button
                    top_bar
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.2, 0.2)),
                            BackToMenuButton,
                        ))
                        .with_child((
                            Text::new("← Menu"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                    // Level name
                    top_bar.spawn((
                        Text::new(level.name.clone()),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Reset button
                    top_bar
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                            ResetButton,
                        ))
                        .with_child((
                            Text::new("Reset"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                });
        });
}

pub fn handle_back_button(
    interaction_query: Query<&Interaction, (With<BackToMenuButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Menu);
        }
    }
}

pub fn handle_reset_button(
    interaction_query: Query<&Interaction, (With<ResetButton>, Changed<Interaction>)>,
    mut mirror_query: Query<(
        &mut crate::mirror::MirrorOrientation,
        &crate::mirror::OriginalOrientation,
        &mut Transform,
    )>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for (mut orientation, original, mut transform) in mirror_query.iter_mut() {
                *orientation = original.0;
                transform.rotation = Quat::from_rotation_z(original.0.rotation_radians());
            }
        }
    }
}

pub fn despawn_hud(mut commands: Commands, hud_query: Query<Entity, With<HudRoot>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn();
    }
}

// ─── Win Overlay ────────────────────────────────────────────

#[derive(Component)]
pub struct WinOverlay;

#[derive(Component)]
pub struct WinBackButton;

pub fn check_win_condition(
    mut commands: Commands,
    level_solved: Res<LevelSolved>,
    existing_overlay: Query<Entity, With<WinOverlay>>,
) {
    if !level_solved.is_changed() {
        return;
    }

    // Remove existing overlay if any
    for entity in existing_overlay.iter() {
        commands.entity(entity).despawn();
    }

    if level_solved.0 {
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                Pickable::IGNORE,
                WinOverlay,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(40.0)),
                            row_gap: Val::Px(20.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.3, 0.15)),
                        Pickable::IGNORE,
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new("Level Complete!"),
                            TextFont {
                                font_size: 42.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 1.0, 0.4)),
                        ));

                        panel
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
                                WinBackButton,
                            ))
                            .with_child((
                                Text::new("Back to Menu"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                    });
            });
    }
}

pub fn handle_win_back_button(
    interaction_query: Query<&Interaction, (With<WinBackButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Menu);
        }
    }
}

pub fn despawn_win_overlay(mut commands: Commands, overlay_query: Query<Entity, With<WinOverlay>>) {
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn();
    }
}
