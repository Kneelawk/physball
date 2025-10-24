use crate::game::gui::{ButtonSettings, button, menu_root, title};
use crate::game::levels::SelectedLevel;
use crate::game::menus::options_menu::OptionsReturn;
use crate::game::state::AppState;
use bevy::prelude::*;
use bevy::ui_widgets::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(AppState::MainMenu), set_main_menu)
            .add_systems(OnExit(AppState::MainMenu), disable_main_menu)
            .add_systems(OnEnter(MenuState::Main), setup_main_menu)
            .add_systems(OnEnter(MenuState::LevelSelect), setup_level_select);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub enum MenuState {
    #[default]
    Disabled,
    Main,
    LevelSelect,
    Options,
}

fn set_main_menu(state: Res<State<MenuState>>, mut next_menu: ResMut<NextState<MenuState>>) {
    if *state == MenuState::Disabled {
        next_menu.set(MenuState::Main);
    }
}

fn disable_main_menu(mut next_menu: ResMut<NextState<MenuState>>) {
    next_menu.set(MenuState::Disabled);
}

fn setup_main_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        menu_root(MenuState::Main),
        children![
            (
                title(&asset_server, "physball"),
                Node {
                    bottom: px(100),
                    ..default()
                }
            ),
            (
                button(&asset_server, "Level Select", default()),
                observe(
                    |_a: On<Activate>, mut next_menu: ResMut<NextState<MenuState>>| {
                        next_menu.set(MenuState::LevelSelect);
                    }
                )
            ),
            (
                button(&asset_server, "Options", default()),
                observe(
                    |_a: On<Activate>,
                     mut cmd: Commands,
                     mut next_menu: ResMut<NextState<MenuState>>| {
                        cmd.insert_resource(OptionsReturn::MainMenu);
                        next_menu.set(MenuState::Options);
                    }
                )
            ),
            (
                button(&asset_server, "Quit", default()),
                observe(|_a: On<Activate>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                })
            )
        ],
    ));
}

fn setup_level_select(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        menu_root(MenuState::LevelSelect),
        children![
            (
                title(&asset_server, "Level Select"),
                Node {
                    bottom: px(100),
                    ..default()
                }
            ),
            (
                Node {
                    width: percent(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: px(20),
                    column_gap: px(20),
                    ..default()
                },
                children![
                    (
                        button(&asset_server, "Level 1", ButtonSettings::level_select()),
                        observe(
                            |_a: On<Activate>,
                             mut next_state: ResMut<NextState<AppState>>,
                             mut cmd: Commands| {
                                next_state.set(AppState::Game);
                                cmd.insert_resource(SelectedLevel::Level1);
                            }
                        )
                    ),
                    (
                        button(&asset_server, "Level 2", ButtonSettings::level_select()),
                        observe(
                            |_a: On<Activate>,
                             mut next_state: ResMut<NextState<AppState>>,
                             mut cmd: Commands| {
                                next_state.set(AppState::Game);
                                cmd.insert_resource(SelectedLevel::Level2);
                            }
                        )
                    )
                ]
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
