use crate::game::assets::fonts::BuiltinFonts;
use bevy::input_focus::tab_navigation::{TabGroup, TabIndex};
use bevy::input_focus::{InputFocus, IsFocused, IsFocusedHelper};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::reflect::Is;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::{Button, Slider, SliderRange, SliderThumb, SliderValue};

pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const BUTTON_BG_DEFAULT: Color = Color::srgb(0.2, 0.2, 0.2);
pub const BUTTON_BG_DISABLED: Color = Color::srgb(0.3, 0.3, 0.3);
pub const BUTTON_BG_HOVERED: Color = Color::srgb(0.4, 0.4, 0.4);
pub const BUTTON_BG_PRESSED: Color = Color::srgb(0.1, 0.1, 0.1);
pub const BUTTON_BORDER: Color = Color::srgb(0.6, 0.6, 0.6);
pub const BUTTON_BORDER_FOCUSED: Color = Color::srgb(0.0, 0.8, 0.9);
pub const SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);
pub const SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
pub const SLIDER_THUMB_DISABLED: Color = Color::srgb(0.5, 0.5, 0.5);

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct GuiPlugin;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct MenuButton;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct MenuSlider;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct MenuSliderThumb;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(button_on_interaction::<Add, Pressed>)
            .add_observer(button_on_interaction::<Remove, Pressed>)
            .add_observer(button_on_interaction::<Add, InteractionDisabled>)
            .add_observer(button_on_interaction::<Remove, InteractionDisabled>)
            .add_observer(button_on_interaction::<Insert, Hovered>)
            .add_systems(Update, on_focus_outline::<MenuButton>)
            .add_observer(slider_on_interaction::<Add, InteractionDisabled>)
            .add_observer(slider_on_interaction::<Remove, InteractionDisabled>)
            .add_observer(slider_on_interaction::<Insert, Hovered>)
            .add_observer(slider_on_change_value::<SliderValue>)
            .add_observer(slider_on_change_value::<SliderRange>)
            .add_systems(Update, on_focus_outline::<MenuSlider>);
    }
}

pub fn menu_root<S: States>(menu_state: S) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(25),
            ..default()
        },
        TabGroup::default(),
        DespawnOnExit(menu_state),
    )
}

pub fn title(fonts: &BuiltinFonts, text: impl ToString) -> impl Bundle {
    (
        Text::new(text.to_string()),
        TextFont {
            font: fonts.title.clone(),
            font_size: 45.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    )
}

pub struct ButtonSettings {
    width: Val,
    height: Val,
}

impl Default for ButtonSettings {
    fn default() -> Self {
        Self {
            width: px(300),
            height: px(65),
        }
    }
}

impl ButtonSettings {
    pub fn small() -> Self {
        Self {
            width: auto(),
            height: px(65),
        }
    }
}

pub fn button<T: ToString>(
    fonts: &BuiltinFonts,
    text: T,
    settings: ButtonSettings,
) -> impl Bundle + use<T> {
    (
        Node {
            width: settings.width,
            height: settings.height,
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(15)),
            ..default()
        },
        Button,
        MenuButton,
        Hovered::default(),
        TabIndex(0),
        BorderColor::all(BUTTON_BORDER),
        BackgroundColor(BUTTON_BG_DEFAULT),
        children![(
            Text::new(text.to_string()),
            TextFont {
                font: fonts.text.clone(),
                font_size: 30.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    )
}

// Copied from Bevy Standard Widgets examples
fn button_on_interaction<E: EntityEvent, C: Bundle>(
    event: On<E, C>,
    mut buttons: Query<
        (
            &Hovered,
            Has<InteractionDisabled>,
            Has<Pressed>,
            &mut BackgroundColor,
            &Children,
        ),
        With<MenuButton>,
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

// Copied from Bevy Tab Navigation example
fn on_focus_outline<T: Component>(
    mut cmd: Commands,
    focus: Res<InputFocus>,
    focus_helper: IsFocusedHelper,
    query: Query<Entity, With<T>>,
) {
    if focus.is_changed() {
        for button in query {
            if focus_helper.is_focus_visible(button) {
                cmd.entity(button).insert(Outline {
                    color: BUTTON_BORDER_FOCUSED,
                    width: px(2),
                    offset: px(0),
                });
            } else {
                cmd.entity(button).remove::<Outline>();
            }
        }
    }
}

/// Copied from Bevy standard widgets example
pub fn slider(min: f32, max: f32, value: f32) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            column_gap: px(4),
            height: px(12),
            width: percent(100),
            ..default()
        },
        Name::new("Slider"),
        Hovered::default(),
        MenuSlider,
        Slider::default(),
        SliderValue(value),
        SliderRange::new(min, max),
        TabIndex(0),
        Children::spawn((
            // Slider background rail
            Spawn((
                Node {
                    height: px(6),
                    ..default()
                },
                BackgroundColor(SLIDER_TRACK), // Border color for the checkbox
                BorderRadius::all(px(3)),
            )),
            // Invisible track to allow absolute placement of thumb entity. This is narrower than
            // the actual slider, which allows us to position the thumb entity using simple
            // percentages, without having to measure the actual width of the slider thumb.
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    // Track is short by 12px to accommodate the thumb.
                    right: px(12),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                children![(
                    // Thumb
                    MenuSliderThumb,
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: px(12),
                        height: px(12),
                        position_type: PositionType::Absolute,
                        left: percent(0), // This will be updated by the slider's value
                        ..default()
                    },
                    BorderRadius::MAX,
                    BackgroundColor(SLIDER_THUMB),
                )],
            )),
        )),
    )
}

/// Copied from Bevy standard widgets example
fn slider_on_interaction<E: EntityEvent, C: Component>(
    event: On<E, C>,
    sliders: Query<(Entity, &Hovered, Has<InteractionDisabled>), With<MenuSlider>>,
    children: Query<&Children>,
    mut thumbs: Query<(&mut BackgroundColor, Has<MenuSliderThumb>), Without<MenuSlider>>,
) {
    if let Ok((slider_ent, hovered, disabled)) = sliders.get(event.event_target()) {
        // These "removal event checks" exist because the `Remove` event is triggered _before_ the component is actually
        // removed, meaning it still shows up in the query. We're investigating the best way to improve this scenario.
        let disabled = disabled && !(E::is::<Remove>() && C::is::<InteractionDisabled>());
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_bg, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                thumb_bg.0 = thumb_color(disabled, hovered.0);
            }
        }
    }
}

/// Copied from Bevy standard widgets example
fn slider_on_change_value<C: Component>(
    insert: On<Insert, C>,
    sliders: Query<(Entity, &SliderValue, &SliderRange), With<MenuSlider>>,
    children: Query<&Children>,
    mut thumbs: Query<(&mut Node, Has<MenuSliderThumb>), Without<MenuSlider>>,
) {
    if let Ok((slider_ent, value, range)) = sliders.get(insert.entity) {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                thumb_node.left = percent(range.thumb_position(value.0) * 100.0);
            }
        }
    }
}

/// Copied from Bevy standard widgets example
fn thumb_color(disabled: bool, hovered: bool) -> Color {
    match (disabled, hovered) {
        (true, _) => SLIDER_THUMB_DISABLED,

        (false, true) => SLIDER_THUMB.lighter(0.3),

        _ => SLIDER_THUMB,
    }
}
