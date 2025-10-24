use crate::game::gui::{button, menu_root, title};
use crate::game::menus::main_menu::MenuState;
use crate::game::menus::pause_menu::PauseMenuState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct OptionsMenuPlugin;

impl Plugin for OptionsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OptionsMenuState>()
            .add_systems(OnEnter(MenuState::Options), set_options_menu)
            .add_systems(OnExit(MenuState::Options), disable_options_menu)
            .add_systems(OnEnter(PauseMenuState::Options), set_options_menu)
            .add_systems(OnExit(PauseMenuState::Options), disable_options_menu)
            .add_systems(OnEnter(OptionsMenuState::Main), setup_main_options_menu);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub enum OptionsMenuState {
    #[default]
    Disabled,
    Main,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Resource)]
pub enum OptionsReturn {
    MainMenu,
    PauseMenu,
}

fn set_options_menu(mut next_state: ResMut<NextState<OptionsMenuState>>) {
    next_state.set(OptionsMenuState::Main);
}

fn disable_options_menu(mut next_state: ResMut<NextState<OptionsMenuState>>) {
    next_state.set(OptionsMenuState::Disabled);
}

fn setup_main_options_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        menu_root(OptionsMenuState::Main),
        children![
            (
                title(&asset_server, "Options"),
                Node {
                    bottom: px(50),
                    ..default()
                }
            ),
            (
                button(&asset_server, "Back", default()),
                observe(
                    |_a: On<Activate>,
                     ret: Res<OptionsReturn>,
                     mut next_menu: ResMut<NextState<MenuState>>,
                     mut next_pause: ResMut<NextState<PauseMenuState>>| {
                        match *ret {
                            OptionsReturn::MainMenu => next_menu.set(MenuState::Main),
                            OptionsReturn::PauseMenu => next_pause.set(PauseMenuState::Main),
                        }
                    }
                )
            )
        ],
    ));
}
