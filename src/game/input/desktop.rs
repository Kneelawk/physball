use super::PlayerInput;
use crate::game::game_state::GameState;
use crate::game::settings::GamePrefs;
use bevy::input::keyboard::Key;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

#[allow(dead_code)]
pub fn build(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (
            pause_play,
            fn_key_input,
            keyboard_input,
            mouse_input,
            mouse_scroll,
        ),
    );
}

#[allow(dead_code)]
pub fn game_started(state: &mut NextState<GameState>) {
    state.set(GameState::Playing);
}

#[allow(dead_code)]
pub fn pause_play(mut writer: MessageWriter<PlayerInput>, input: Res<ButtonInput<Key>>) {
    if input.just_pressed(Key::Escape) {
        writer.write(PlayerInput::Pause { toggle: true });
    }
}

pub fn fn_key_input(mut writer: MessageWriter<PlayerInput>, input: Res<ButtonInput<Key>>) {
    if input.just_pressed(Key::F3) {
        writer.write(PlayerInput::ToggleGizmos);
    }

    if input.just_pressed(Key::F2) {
        writer.write(PlayerInput::Screenshot);
    }
}

pub fn keyboard_input(mut writer: MessageWriter<PlayerInput>, input: Res<ButtonInput<KeyCode>>) {
    let mut movement = Vec2::default();
    if input.pressed(KeyCode::KeyW) {
        movement += Vec2::Y;
    }
    if input.pressed(KeyCode::KeyS) {
        movement -= Vec2::Y;
    }
    if input.pressed(KeyCode::KeyA) {
        movement -= Vec2::X;
    }
    if input.pressed(KeyCode::KeyD) {
        movement += Vec2::X;
    }

    writer.write(PlayerInput::Movement(movement.normalize_or_zero()));

    if input.pressed(KeyCode::Space) {
        writer.write(PlayerInput::Jump);
    }
}

pub fn mouse_input(
    mut writer: MessageWriter<PlayerInput>,
    mut mouse: MessageReader<MouseMotion>,
    prefs: Res<GamePrefs>,
) {
    let mouse_speed = prefs.mouse_speed / 1000.0;
    for mouse in mouse.read() {
        writer.write(PlayerInput::CameraMovement(mouse.delta * mouse_speed));
    }
}

pub fn mouse_scroll(mut writer: MessageWriter<PlayerInput>, mut scroll: MessageReader<MouseWheel>) {
    for scroll in scroll.read() {
        writer.write(PlayerInput::Zoom(scroll.y));
    }
}
