// desktop module is used by most of the other modules
mod desktop;
mod gamepad;
#[cfg(feature = "input-web")]
mod web;

use crate::game::game_state::GameState;
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerInput>();

        #[cfg(feature = "input-desktop")]
        desktop::build(app);

        #[cfg(feature = "input-web")]
        web::build(app);

        gamepad::build(app);
    }
}

pub fn game_started(state: &mut NextState<GameState>) {
    #[cfg(not(any(feature = "input-desktop", feature = "input-web")))]
    compile_error!("Either 'input-desktop' or 'input-web' feature must be enabled");

    #[cfg(feature = "input-desktop")]
    desktop::game_started(state);

    #[cfg(feature = "input-web")]
    web::game_started(state);
}

#[derive(Debug, Copy, Clone, PartialEq, Message, Reflect)]
#[reflect(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PlayerInput {
    CameraMovement(Vec2),
    Zoom(f32),
    Movement(Vec2),
    Jump,
    Pause { toggle: bool },
    ToggleGizmos,
    Screenshot,
}
