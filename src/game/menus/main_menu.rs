use crate::game::gui::{ButtonSettings, button, menu_root, title};
use crate::game::levels::SelectedLevel;
use crate::game::levels::index::{LevelIndex, LevelIndexAsset, LevelRef};
use crate::game::menus::options_menu::OptionsReturn;
use crate::game::state::AppState;
use bevy::prelude::*;
use bevy::ui_widgets::*;
use crate::game::assets::preload::Preloads;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(AppState::MainMenu), set_main_menu)
            .add_systems(OnExit(AppState::MainMenu), disable_main_menu)
            .add_systems(OnEnter(MenuState::Main), setup_main_menu)
            .add_systems(OnEnter(MenuState::LevelSelect), setup_level_select)
            .add_systems(
                PreUpdate,
                on_level_index_change.run_if(in_state(MenuState::LevelSelect)),
            );
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

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct LevelSelectMenu;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Component)]
pub struct LevelSelectButton(String);

fn set_main_menu(state: Res<State<MenuState>>, mut next_menu: ResMut<NextState<MenuState>>) {
    if *state == MenuState::Disabled {
        next_menu.set(MenuState::Main);
    }
}

fn disable_main_menu(mut next_menu: ResMut<NextState<MenuState>>) {
    next_menu.set(MenuState::Disabled);
}

fn setup_main_menu(mut cmd: Commands, fonts: Res<Preloads>) {
    cmd.spawn((
        menu_root(MenuState::Main),
        children![
            (
                title(&fonts, "physball"),
                Node {
                    bottom: vh(10),
                    ..default()
                }
            ),
            (
                button(&fonts, "Level Select", default()),
                observe(
                    |_a: On<Activate>, mut next_menu: ResMut<NextState<MenuState>>| {
                        next_menu.set(MenuState::LevelSelect);
                    }
                )
            ),
            (
                button(&fonts, "Options", default()),
                observe(
                    |_a: On<Activate>,
                     mut cmd: Commands,
                     mut next_menu: ResMut<NextState<MenuState>>| {
                        cmd.insert_resource(OptionsReturn::MainMenu);
                        next_menu.set(MenuState::Options);
                    }
                )
            ),
            {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    (
                        button(&fonts, "Quit", default()),
                        observe(|_a: On<Activate>, mut exit: MessageWriter<AppExit>| {
                            exit.write(AppExit::Success);
                        }),
                    )
                }
            }
        ],
    ));
}

fn on_level_index_change(
    mut msg: MessageReader<AssetEvent<LevelIndex>>,
    mut cmd: Commands,
    menu: Query<Entity, With<LevelSelectMenu>>,
    fonts: Res<Preloads>,
    index_handle: Res<LevelIndexAsset>,
    index: Res<Assets<LevelIndex>>,
) {
    for e in msg.read() {
        if e.is_loaded_with_dependencies(&index_handle.0) {
            // despawn the old menu
            for entity in menu {
                cmd.entity(entity).despawn();
            }

            setup_level_select(cmd, fonts, index_handle, index);

            msg.clear();
            return;
        }
    }
}

fn setup_level_select(
    mut cmd: Commands,
    fonts: Res<Preloads>,
    index_handle: Res<LevelIndexAsset>,
    index: Res<Assets<LevelIndex>>,
) {
    let level_buttons = index
        .get(&index_handle.0)
        .iter()
        .flat_map(|idx| idx.order.iter().map(|name| &idx.levels[name]))
        .map(|r| level_select_button(&fonts, r))
        .collect::<Vec<_>>();

    cmd.spawn((
        menu_root(MenuState::LevelSelect),
        LevelSelectMenu,
        children![
            (
                title(&fonts, "Level Select"),
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
                Children::spawn(SpawnIter(level_buttons.into_iter())),
            ),
            (
                button(&fonts, "Back", default()),
                observe(
                    |_a: On<Activate>, mut next_menu: ResMut<NextState<MenuState>>| {
                        next_menu.set(MenuState::Main);
                    }
                )
            )
        ],
    ));
}

fn level_select_button(fonts: &Preloads, level: &LevelRef) -> impl Bundle + use<> {
    let display = level.display.clone();
    let name = level.name.clone();

    // we use LevelSelectButton to pass data into the observe closure because there's a bug in
    // observe closures that strips them of their extra moved data
    (
        button(fonts, display, ButtonSettings::small()),
        LevelSelectButton(name),
        observe(
            |a: On<Activate>,
             mut next_state: ResMut<NextState<AppState>>,
             button: Query<&LevelSelectButton>,
             mut cmd: Commands| {
                let name = button
                    .get(a.entity)
                    .expect("level select button missing LevelSelectButton component")
                    .0
                    .clone();
                next_state.set(AppState::LoadingLevel);
                cmd.insert_resource(SelectedLevel(name.clone()));
            },
        ),
    )
}
