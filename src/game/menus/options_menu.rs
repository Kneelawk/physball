use crate::game::assets::fonts::BuiltinFonts;
use crate::game::gui::{ButtonSettings, TEXT_COLOR, button, menu_root, slider, title};
use crate::game::menus::main_menu::MenuState;
use crate::game::menus::pause_menu::PauseMenuState;
use crate::game::settings::{DEFAULT_MOUSE_SPEED, GamePrefs};
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
            .add_systems(Update, update_mouse_speed_slider)
            .add_systems(Update, update_mouse_speed_text);
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

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
pub struct MouseSpeedText;

fn set_options_menu(mut next_state: ResMut<NextState<OptionsMenuState>>) {
    next_state.set(OptionsMenuState::Main);
}

fn disable_options_menu(mut next_state: ResMut<NextState<OptionsMenuState>>) {
    next_state.set(OptionsMenuState::Disabled);
}

fn setup_main_options_menu(mut cmd: Commands, fonts: Res<BuiltinFonts>, prefs: Res<GamePrefs>) {
    info!("mouse speed: {}", prefs.mouse_speed);
    cmd.spawn((
        menu_root(OptionsMenuState::Main),
        children![
            (
                title(&fonts, "Options"),
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
                    row_gap: px(20),
                    ..default()
                },
                children![
                    window_resize_section(&fonts),
                    mouse_sensitivity_section(&fonts, &prefs),
                ]
            ),
            (
                button(&fonts, "Back", default()),
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

fn mouse_sensitivity_section(fonts: &BuiltinFonts, prefs: &GamePrefs) -> impl Bundle {
    (
        Node {
            width: percent(100),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        children![
            (
                Node {
                    width: percent(100),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::SpaceBetween,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                children![
                    (
                        Text::new("Mouse Sensitivity"),
                        TextFont {
                            font: fonts.text.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ),
                    (
                        MouseSpeedText,
                        Text::new(format!("{:.2}", prefs.mouse_speed)),
                        TextFont {
                            font: fonts.text.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    )
                ]
            ),
            (
                Node {
                    width: percent(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: px(10),
                    ..default()
                },
                children![
                    (
                        slider(0.0, 10.0, prefs.mouse_speed),
                        MouseSpeedSlider,
                        observe(
                            |value_change: On<ValueChange<f32>>, mut prefs: ResMut<GamePrefs>| {
                                prefs.mouse_speed = value_change.value;
                                prefs.save();
                            }
                        )
                    ),
                    (
                        button(fonts, "Reset", ButtonSettings::small()),
                        observe(|_on: On<Activate>, mut prefs: ResMut<GamePrefs>| {
                            prefs.mouse_speed = DEFAULT_MOUSE_SPEED;
                            prefs.save();
                        })
                    )
                ]
            )
        ],
    )
}

fn window_resize_section(fonts: &BuiltinFonts) -> impl Bundle {
    (
        Node {
            width: percent(100),
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
                    font: fonts.text.clone(),
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
                    window_resize_button(fonts, 960, 540),
                    window_resize_button(fonts, 1280, 720),
                    window_resize_button(fonts, 1920, 1080),
                    window_resize_button(fonts, 2048, 1152),
                    window_resize_button(fonts, 2560, 1440),
                    window_resize_button(fonts, 3840, 2160),
                ],
            ),
        ],
    )
}

fn window_resize_button(fonts: &BuiltinFonts, width: u32, height: u32) -> impl Bundle {
    (
        button(
            fonts,
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

fn update_mouse_speed_text(
    prefs: Res<GamePrefs>,
    slider: Single<Entity, With<MouseSpeedText>>,
    mut cmd: Commands,
) {
    if prefs.is_changed() {
        cmd.entity(*slider)
            .insert(Text::new(format!("{:.2}", prefs.mouse_speed)));
    }
}
