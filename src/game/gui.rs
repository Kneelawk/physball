use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::reflect::Is;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;

pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const BUTTON_BG_DEFAULT: Color = Color::srgb(0.2, 0.2, 0.2);
pub const BUTTON_BG_DISABLED: Color = Color::srgb(0.3, 0.3, 0.3);
pub const BUTTON_BG_HOVERED: Color = Color::srgb(0.4, 0.4, 0.4);
pub const BUTTON_BG_PRESSED: Color = Color::srgb(0.1, 0.1, 0.1);
pub const BUTTON_BORDER: Color = Color::srgb(0.6, 0.6, 0.6);

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(button_on_interaction::<Add, Pressed>)
            .add_observer(button_on_interaction::<Remove, Pressed>)
            .add_observer(button_on_interaction::<Add, InteractionDisabled>)
            .add_observer(button_on_interaction::<Remove, InteractionDisabled>)
            .add_observer(button_on_interaction::<Insert, Hovered>);
    }
}

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

// Copied from Bevy Standard Widgets examples
fn button_on_interaction<E: EntityEvent, C: Component>(
    event: On<E, C>,
    mut buttons: Query<
        (
            &Hovered,
            Has<InteractionDisabled>,
            Has<Pressed>,
            &mut BackgroundColor,
            &Children,
        ),
        With<Button>,
    >,
) {
    if let Ok((hovered, disabled, pressed, mut color, children)) =
        buttons.get_mut(event.event_target())
    {
        if children.is_empty() {
            return;
        }
        let hovered = hovered.get();
        // These "removal event checks" exist because the `Remove` event is triggered _before_ the component is actually
        // removed, meaning it still shows up in the query. We're investigating the best way to improve this scenario.
        let pressed = pressed && !(E::is::<Remove>() && C::is::<Pressed>());
        let disabled = disabled && !(E::is::<Remove>() && C::is::<InteractionDisabled>());
        match (disabled, hovered, pressed) {
            // Disabled button
            (true, _, _) => {
                *color = BUTTON_BG_DISABLED.into();
            }

            // Pressed and hovered button
            (false, true, true) => {
                *color = BUTTON_BG_PRESSED.into();
            }

            // Hovered, unpressed button
            (false, true, false) => {
                *color = BUTTON_BG_HOVERED.into();
            }

            // Unhovered button (either pressed or not).
            (false, false, _) => {
                *color = BUTTON_BG_DEFAULT.into();
            }
        }
    }
}
