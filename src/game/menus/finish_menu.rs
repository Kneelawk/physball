use crate::game::game_state::GameState;
use crate::game::gui::{button, menu_root, title};
use crate::game::menus::main_menu::MenuState;
use crate::game::state::AppState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FinishMenuPlugin;

impl Plugin for FinishMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<FinishMenuState>()
            .add_systems(OnEnter(GameState::Finished), set_finish_menu)
            .add_systems(OnExit(GameState::Finished), disable_finish_menu)
            .add_systems(OnEnter(FinishMenuState::Main), setup_finish_menu);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub enum FinishMenuState {
    #[default]
    Disabled,
    Main,
}

fn set_finish_menu(mut next_state: ResMut<NextState<FinishMenuState>>) {
    next_state.set(FinishMenuState::Main);
}

fn disable_finish_menu(mut next_state: ResMut<NextState<FinishMenuState>>) {
    next_state.set(FinishMenuState::Disabled);
}

fn setup_finish_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        menu_root(FinishMenuState::Main),
        children![
            (
                title(&asset_server, "Level Finished"),
                Node {
                    bottom: px(50),
                    ..default()
                }
            ),
            (
                button(&asset_server, "Exit Level", default()),
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
