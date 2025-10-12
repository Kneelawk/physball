use crate::game::gui::{button, title};
use crate::game::state::GameState;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::ui_widgets::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(GameState::MainMenu), set_main_menu)
            .add_systems(OnEnter(MenuState::Main), setup_main_menu);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States)]
pub enum MenuState {
    #[default]
    Disabled,
    Main,
}

fn set_main_menu(mut next_menu: ResMut<NextState<MenuState>>) {
    next_menu.set(MenuState::Main);
}

fn setup_main_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        TabGroup::default(),
        children![
            (
                title(&asset_server, "physball"),
                Node {
                    bottom: px(100),
                    ..default()
                }
            ),
            (
                button(&asset_server, "Level Select"),
                observe(|_activate: On<Activate>| {
                    info!("TODO: Level Select");
                })
            ),
            (
                button(&asset_server, "Quit"),
                observe(
                    |_activate: On<Activate>, mut exit: MessageWriter<AppExit>| {
                        exit.write(AppExit::Success);
                    }
                )
            )
        ],
    ));
}
