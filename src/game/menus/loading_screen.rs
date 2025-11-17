use crate::game::assets::preload::Preloads;
use crate::game::gui::{TEXT_COLOR, button, menu_root};
use crate::game::levels::index::LevelIndex;
use crate::game::levels::{LevelHandle, LevelLoadingLock, SelectedLevel};
use crate::game::menus::main_menu::MenuState;
use crate::game::state::AppState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};

const ERROR_TEXT_COLOR: Color = Color::srgb(0.9, 0.0, 0.0);

#[derive(Debug, Default)]
pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            build_loading_screen(
                "Loading Level...",
                "Loading level",
                "",
                "Cancel",
                false,
                AppState::LoadingLevel,
                false,
            ),
        )
        .add_systems(
            OnEnter(AppState::LevelLoadingError),
            build_loading_screen(
                "Error Loading Level",
                "Error loading level",
                ". Check the console for details.",
                "Return to level select screen",
                true,
                AppState::LevelLoadingError,
                true,
            ),
        );
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Message, Reflect)]
#[reflect(Debug, Default, Clone, Hash)]
struct CancelLevelLoading;

fn build_loading_screen(
    title_str: &str,
    loading_level_str: &str,
    loading_level_suffix: &str,
    cancel_str: &str,
    error_title_color: bool,
    state: AppState,
    ignore_loading_lock: bool,
) -> impl FnMut(Commands, Res<Preloads>, Option<Res<SelectedLevel>>, Res<LevelIndex>) {
    move |mut cmd: Commands,
          preloads: Res<Preloads>,
          selected_level: Option<Res<SelectedLevel>>,
          level_index: Res<LevelIndex>| {
        let level_name = selected_level
            .and_then(|level| level_index.levels.get(&level.0))
            .map(|level| &level.display as &str)
            .unwrap_or("null");

        cmd.spawn((
            menu_root(state),
            children![
                (
                    Text::new(title_str.to_string()),
                    TextFont {
                        font: preloads.title_font(),
                        font_size: 45.0,
                        ..default()
                    },
                    TextColor(if error_title_color {
                        ERROR_TEXT_COLOR
                    } else {
                        TEXT_COLOR
                    }),
                ),
                (
                    Text(format!(
                        "{} '{}'{}",
                        loading_level_str, level_name, loading_level_suffix
                    )),
                    TextFont {
                        font: preloads.text_font(),
                        font_size: 32.0,
                        ..default()
                    }
                ),
                (
                    button(&preloads, cancel_str, default()),
                    observe(
                        move |_on: On<Activate>,
                              mut cmd: Commands,
                              mut next_state: ResMut<NextState<AppState>>,
                              mut next_menu: ResMut<NextState<MenuState>>,
                              mut level_lock: ResMut<LevelLoadingLock>| {
                            if ignore_loading_lock || *level_lock == LevelLoadingLock::Loading {
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
}
