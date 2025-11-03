use crate::game::gui::{ButtonSettings, TEXT_COLOR, button, menu_root, slider, title};
use crate::game::menus::main_menu::MenuState;
use crate::game::menus::pause_menu::PauseMenuState;
use crate::game::settings::GamePrefs;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, SliderValue, ValueChange, observe};

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct OptionsMenuPlugin;

impl Plugin for OptionsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OptionsMenuState>()
            .add_systems(OnEnter(MenuState::Options), set_options_menu)
            .add_systems(OnExit(MenuState::Options), disable_options_menu)
            .add_systems(OnEnter(PauseMenuState::Options), set_options_menu)
            .add_systems(OnExit(PauseMenuState::Options), disable_options_menu)
            .add_systems(OnEnter(OptionsMenuState::Main), setup_main_options_menu)
            .add_systems(Update, update_mouse_speed_slider);
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

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
pub struct MouseSpeedSlider;

fn set_options_menu(mut next_state: ResMut<NextState<OptionsMenuState>>) {
    next_state.set(OptionsMenuState::Main);
}

fn disable_options_menu(mut next_state: ResMut<NextState<OptionsMenuState>>) {
    next_state.set(OptionsMenuState::Disabled);
}

fn setup_main_options_menu(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    prefs: Res<GamePrefs>,
) {
    info!("mouse speed: {}", prefs.mouse_speed);
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
                Node {
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Center,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                children![
                    (
                        Text::new("Window Resolution"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ),
                    (
                        Node {
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: px(10),
                            column_gap: px(10),
                            ..default()
                        },
                        children![
                            window_resize_button(&asset_server, 960, 540),
                            window_resize_button(&asset_server, 1280, 720),
                            window_resize_button(&asset_server, 1920, 1080),
                            window_resize_button(&asset_server, 2048, 1152),
                            window_resize_button(&asset_server, 2560, 1440),
                            window_resize_button(&asset_server, 3840, 2160),
                        ]
                    ),
                    (
                        Node {
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Center,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(10),
                            width: percent(100),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Mouse Sensitivity"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                                    font_size: 32.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ),
                            (
                                slider(0.0, 10.0, prefs.mouse_speed),
                                MouseSpeedSlider,
                                observe(
                                    |value_change: On<ValueChange<f32>>, mut prefs: ResMut<GamePrefs>| {
                                        prefs.mouse_speed = value_change.value;
                                        prefs.save();
                                    }
                                )
                            )
                        ]
                    )
                ]
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

fn window_resize_button(asset_server: &AssetServer, width: u32, height: u32) -> impl Bundle {
    (
        button(
            asset_server,
            format!("{}x{}", width, height),
            ButtonSettings::small(),
        ),
        observe(
            move |_a: On<Activate>,
                  mut window: Single<&mut Window>,
                  mut prefs: ResMut<GamePrefs>| {
                info!("Resizing window...");
                window.resolution.set(width as f32, height as f32);
                info!("Physical size: {}", window.physical_size());
                prefs.window_width = width;
                prefs.window_height = height;
                prefs.save();
            },
        ),
    )
}

fn update_mouse_speed_slider(
    prefs: Res<GamePrefs>,
    slider: Single<Entity, With<MouseSpeedSlider>>,
    mut cmd: Commands,
) {
    if prefs.is_changed() {
        cmd.entity(*slider).insert(SliderValue(prefs.mouse_speed));
    }
}
