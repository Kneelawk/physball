use crate::game::game_state::GameState;
use crate::game::gui::{button, menu_root, title};
use crate::game::levels::LevelRespawnEvent;
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

fn spawn_pause_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        menu_root(PauseMenuState::Main),
        children![
            (
                title(&asset_server, "Game Paused"),
                Node {
                    bottom: px(50),
                    ..default()
                }
            ),
            (
                button(&asset_server, "Continue", default()),
                observe(
                    |_a: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    }
                )
            ),
            (
                button(&asset_server, "Restart", default()),
                observe(
                    |_a: On<Activate>,
                     mut cmd: Commands,
                     mut next_state: ResMut<NextState<GameState>>| {
                        cmd.trigger(LevelRespawnEvent);
                        next_state.set(GameState::Playing);
                    }
                )
            ),
            (
                button(&asset_server, "Options", default()),
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
                button(&asset_server, "Leave Level", default()),
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
