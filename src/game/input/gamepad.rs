use crate::game::input::PlayerInput;
use crate::game::settings::GamePrefs;
use bevy::prelude::*;

// TODO: UI navigation

pub fn build(app: &mut App) {
    app.add_systems(PreUpdate, (joystick_input, gamepad_buttons));
}

pub fn joystick_input(
    mut writer: MessageWriter<PlayerInput>,
    gamepads: Query<&Gamepad>,
    prefs: Res<GamePrefs>,
) {
    let mut look = Vec2::default();
    let mut move_ = Vec2::default();

    for gamepad in gamepads {
        look += gamepad.right_stick();
        move_ += gamepad.left_stick();
    }

    look.y = -look.y;

    if move_.length_squared() > 1.0 {
        move_ = move_.normalize_or_zero();
    }

    let look_speed = prefs.gamepad_look_speed / 1000.0;
    writer.write(PlayerInput::CameraMovement(look * look_speed));
    writer.write(PlayerInput::Movement(move_));
}

pub fn gamepad_buttons(mut writer: MessageWriter<PlayerInput>, gamepads: Query<&Gamepad>) {
    for gamepad in gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            writer.write(PlayerInput::Jump);
        }
        if gamepad.just_pressed(GamepadButton::Start) {
            writer.write(PlayerInput::Pause { toggle: true });
        }
    }
}
