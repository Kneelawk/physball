use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const BUTTON_BG_DEFAULT: Color = Color::srgb(0.2, 0.2, 0.2);
pub const BUTTON_BORDER: Color = Color::srgb(0.6, 0.6, 0.6);

pub fn title(asset_server: &AssetServer, text: impl ToString) -> impl Bundle {
    (
        Text::new(text.to_string()),
        TextFont {
            font: asset_server.load("fonts/BBHSansBartle-Regular.ttf"),
            font_size: 45.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    )
}

pub fn button(asset_server: &AssetServer, text: impl ToString) -> impl Bundle {
    (
        Node {
            width: px(300),
            height: px(65),
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Button,
        Hovered::default(),
        TabIndex(0),
        BorderColor::all(BUTTON_BORDER),
        BackgroundColor(BUTTON_BG_DEFAULT),
        children![(
            Text::new(text.to_string()),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    )
}
