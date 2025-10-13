use crate::game::gui::{button, menu_root, title};
use crate::game::menus::main_menu::MenuState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct OptionsMenuPlugin;

impl Plugin for OptionsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OptionsMenuState>()
            .add_systems(OnEnter(MenuState::Options), set_options_menu)
            .add_systems(OnExit(MenuState::Options), disable_options_menu)
            .add_systems(OnEnter(OptionsMenuState::Main), setup_main_options_menu);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States)]
pub enum OptionsMenuState {
    #[default]
    Disabled,
    Main,
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
                    |_a: On<Activate>, mut next_menu: ResMut<NextState<MenuState>>| {
                        next_menu.set(MenuState::Main);
                    }
                )
            )
        ],
    ));
}
