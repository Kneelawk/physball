use crate::game::assets::preload::Preloads;
use crate::game::gui::{button, menu_root, title};
use crate::game::levels::index::LevelIndex;
use crate::game::levels::{LevelHandle, LevelLoadingLock, SelectedLevel};
use crate::game::menus::main_menu::MenuState;
use crate::game::state::AppState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};

#[derive(Debug, Default)]
pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadingLevel), show_loading_screen);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Message, Reflect)]
#[reflect(Debug, Default, Clone, Hash)]
struct CancelLevelLoading;

fn show_loading_screen(
    mut cmd: Commands,
    preloads: Res<Preloads>,
    selected_level: Option<Res<SelectedLevel>>,
    level_index: Res<LevelIndex>,
) {
    let level_name = selected_level
        .and_then(|level| level_index.levels.get(&level.0))
        .map(|level| &level.name as &str)
        .unwrap_or("null");

    cmd.spawn((
        menu_root(AppState::LoadingLevel),
        children![
            title(&preloads, "Loading Level..."),
            (
                Text(format!("Loading level {}", level_name)),
                TextFont {
                    font: preloads.text_font(),
                    font_size: 32.0,
                    ..default()
                }
            ),
            (
                button(&preloads, "Cancel", default()),
                observe(
                    |_on: On<Activate>,
                     mut cmd: Commands,
                     mut next_state: ResMut<NextState<AppState>>,
                     mut next_menu: ResMut<NextState<MenuState>>,
                     mut level_lock: ResMut<LevelLoadingLock>| {
                        if *level_lock == LevelLoadingLock::Loading {
                            cmd.remove_resource::<SelectedLevel>();
                            cmd.remove_resource::<LevelHandle>();
                            next_state.set(AppState::MainMenu);
                            next_menu.set(MenuState::LevelSelect);
                            *level_lock = LevelLoadingLock::NotLoading;
                        }
                    }
                )
            )
        ],
    ));
}
