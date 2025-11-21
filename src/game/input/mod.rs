// desktop module is used by most of the other modules
mod desktop;
#[cfg(feature = "input_web")]
mod web;

use crate::game::game_state::GameState;
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerInput>();

        #[cfg(feature = "input_desktop")]
        desktop::build(app);

        #[cfg(feature = "input_web")]
        web::build(app);
    }
}

pub fn game_started(state: &mut NextState<GameState>) {
    #[cfg(not(any(feature = "input_desktop", feature = "input_web")))]
    compile_error!("Either 'input_desktop' or 'input_web' feature must be enabled");

    #[cfg(feature = "input_desktop")]
    desktop::game_started(state);

    #[cfg(feature = "input_web")]
    web::game_started(state);
}

#[derive(Debug, Copy, Clone, PartialEq, Message, Reflect)]
#[reflect(Debug, Clone, PartialEq)]
pub enum PlayerInput {
    CameraMovement(Vec2),
    Zoom(f32),
    Movement(Vec2),
    Jump,
    Pause { toggle: bool },
    ToggleGizmos,
}
