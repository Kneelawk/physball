use crate::game::assets::preload::Preloads;
use crate::game::game_state::GameState;
use crate::game::gui::{button, menu_root, title};
use crate::game::levels::LevelRestartEvent;
use crate::game::menus::main_menu::MenuState;
use crate::game::menus::options_menu::OptionsReturn;
use crate::game::state::AppState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseMenuState>()
            .add_systems(OnEnter(GameState::Paused), set_pause_menu)
            .add_systems(OnExit(GameState::Paused), disable_pause_menu)
            .add_systems(OnEnter(PauseMenuState::Main), spawn_pause_menu);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub enum PauseMenuState {
    #[default]
    Disabled,
    Main,
    Options,
}

fn set_pause_menu(mut next_state: ResMut<NextState<PauseMenuState>>) {
    next_state.set(PauseMenuState::Main);
}

fn disable_pause_menu(mut next_state: ResMut<NextState<PauseMenuState>>) {
    next_state.set(PauseMenuState::Disabled);
}

fn spawn_pause_menu(mut cmd: Commands, fonts: Res<Preloads>) {
    cmd.spawn((
        menu_root(PauseMenuState::Main),
        children![
            (
                title(&fonts, "Game Paused"),
                Node {
                    bottom: px(50),
                    ..default()
                }
            ),
            (
                button(&fonts, "Continue", default()),
                observe(
                    |_a: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    }
                )
            ),
            (
                button(&fonts, "Restart", default()),
                observe(
                    |_a: On<Activate>,
                     mut cmd: Commands,
                     mut next_state: ResMut<NextState<GameState>>| {
                        cmd.trigger(LevelRestartEvent);
                        next_state.set(GameState::Playing);
                    }
                )
            ),
            (
                button(&fonts, "Options", default()),
                observe(
                    |_a: On<Activate>,
                     mut cmd: Commands,
                     mut next_state: ResMut<NextState<PauseMenuState>>| {
                        cmd.insert_resource(OptionsReturn::PauseMenu);
                        next_state.set(PauseMenuState::Options);
                    }
                )
            ),
            (
                button(&fonts, "Exit Level", default()),
                observe(
                    |_a: On<Activate>,
                     mut next_state: ResMut<NextState<AppState>>,
                     mut next_menu: ResMut<NextState<MenuState>>| {
                        next_state.set(AppState::MainMenu);
                        next_menu.set(MenuState::LevelSelect);
                    }
                )
            )
        ],
    ));
}
